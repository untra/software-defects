#!/usr/bin/env python3
"""Drive `state-lens serve` over every corpus file and compare the hints with smoke.json."""
import argparse
import collections
import concurrent.futures
import os
import hashlib
import json
import queue
import subprocess
import sys
import threading
import time
from pathlib import Path

import smoke_report as audit
from corpus_inventory import EXTENSIONS, files, shard as corpus_shard

ROOT = Path(__file__).resolve().parents[3]
CORPUS_ROOTS = [ROOT / "examples", ROOT / "examples-fixed"]
BASELINE = ROOT / "tooling/state-lens/smoke.json"
DEFAULT_BINARY = ROOT / "tooling/state-lens/target/release/state-lens"
SCHEMA_VERSION = 1
MESSAGE_TIMEOUT_S = 5
GROUP_TIMEOUT_S = 10
CONTENT_MODIFIED = -32801
INDEXING_TIMEOUT_S = 30
LOG_ERROR = 1
LOG_WARNING = 2
OPEN_VERSION = 1
RESTORED_VERSION = 3
PUBLISH = "textDocument/publishDiagnostics"
EDITED_VERSION = 2


class Crash(Exception):
    pass


def corpus_files():
    return files()


def relative(path):
    return str(path.relative_to(ROOT))


class Client:
    def __init__(self, binary):
        self.process = subprocess.Popen([str(binary), "serve"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.inbox = queue.Queue()
        self.pending = []
        self.errors = []
        self.latest_diagnostics = {}
        self.next_id = 0
        self.deadline = float("inf")
        self.mutation = 0
        self.requests = {}
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        try:
            stream = self.process.stdout
            while True:
                length = None
                while True:
                    line = stream.readline()
                    if not line:
                        return
                    if not line.strip():
                        break
                    name, _, value = line.decode().partition(":")
                    if name.lower() == "content-length":
                        length = int(value)
                self.inbox.put(json.loads(stream.read(length)))
        except (ValueError, OSError, TypeError) as error:
            self.inbox.put({"reader_error": str(error)})
        finally:
            self.inbox.put(None)

    def send(self, message):
        body = json.dumps({"jsonrpc": "2.0", **message}).encode()
        try:
            self.process.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
            self.process.stdin.flush()
        except (BrokenPipeError, OSError) as error:
            raise Crash(f"server pipe closed (exit {self.process.poll()})") from error

    def notify(self, method, params):
        if method in {"textDocument/didOpen", "textDocument/didChange", "textDocument/didClose"}:
            self.mutation += 1
        self.send({"method": method, "params": params})

    def _receive(self, timeout):
        remaining = min(timeout, self.deadline - time.monotonic())
        if remaining <= 0:
            raise TimeoutError("group deadline exceeded")
        try:
            message = self.inbox.get(timeout=remaining)
        except queue.Empty:
            raise TimeoutError(f"no server message within {remaining:.2f}s") from None
        if message is None or "reader_error" in message:
            raise Crash(f"server stream closed or invalid (exit {self.process.poll()}): {message}")
        if message.get("method") in {"window/logMessage", "window/showMessage"}:
            params = message["params"]
            if params["type"] in (LOG_ERROR, LOG_WARNING):
                self.errors.append(audit.problem("log-error" if params["type"] == LOG_ERROR else "log-warning", params["message"]))
        if message.get("method") == PUBLISH:
            self.latest_diagnostics[message["params"]["uri"]] = message["params"]
        return message

    def begin(self, method, params):
        self.next_id += 1
        self.requests[self.next_id] = (method, self.mutation)
        self.send({"id": self.next_id, "method": method, "params": params})
        return self.next_id

    def response(self, identifier):
        return self.wait_for(lambda m: m.get("id") == identifier)

    def request(self, method, params):
        return self.response(self.begin(method, params))

    def wait_for(self, predicate, timeout=MESSAGE_TIMEOUT_S):
        for index, message in enumerate(self.pending):
            if predicate(message):
                return self.pending.pop(index)
        deadline = min(time.monotonic() + timeout, self.deadline)
        while True:
            message = self._receive(deadline - time.monotonic())
            if predicate(message):
                return message
            self.pending.append(message)

    def shutdown(self):
        self.deadline = time.monotonic() + MESSAGE_TIMEOUT_S
        response = self.request("shutdown", None)
        if "error" in response:
            raise Crash(f"shutdown failed: {response['error']}")
        self.notify("exit", None)
        return self.process.wait(timeout=MESSAGE_TIMEOUT_S)

    def terminate(self):
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait()
        self.process.stdin.close()
        self.process.stdout.close()


def position_key(position):
    return position["line"], position["character"]


def range_problems(name, value, lines):
    start, end = value["start"], value["end"]
    problems = []
    if position_key(start) > position_key(end):
        problems.append(f"{name} starts after it ends")
    for point in (start, end):
        line = point["line"]
        if line < 0 or point["character"] < 0 or line >= len(lines) or point["character"] > len(lines[line].encode("utf-16-le")) // 2:
            problems.append(f"{name} out of bounds at {point}")
    return problems


def summarize(snapshot):
    functions = {json.dumps(e["functionRange"], sort_keys=True) for e in snapshot["findings"] if e["functionRange"]}
    return {
        "source_hash": snapshot["sourceHash"],
        "parse_errors": snapshot["parseErrors"],
        "rule_ids": dict(sorted(collections.Counter(e["finding"]["rule_id"] for e in snapshot["findings"]).items())),
        "hinted_functions": len(functions),
    }


def compare(expected, actual):
    if expected is None:
        return ["missing from smoke.json; run smoke.py --update"]
    if expected["source_hash"] != actual["source_hash"]:
        return ["source changed since smoke.json was generated; run smoke.py --update and review"]
    return [f"{key}: expected {expected[key]}, got {actual[key]}" for key in ("parse_errors", "rule_ids", "hinted_functions") if expected[key] != actual[key]]


def groups(files):
    result = collections.defaultdict(list)
    for path in files:
        result[path.parent].append(path)
    return dict(sorted(result.items()))


def initialize(binary, shard):
    client = Client(binary)
    try:
        response = client.request("initialize", {
            "capabilities": {"textDocument": {"hover": {"contentFormat": ["markdown"]}, "inlayHint": {}, "publishDiagnostics": {"versionSupport": True}}},
            "workspaceFolders": [{"uri": (ROOT / shard).as_uri(), "name": shard}],
            "initializationOptions": {"diagnostics": True},
        })
        if "error" in response:
            raise Crash(str(response["error"]))
        client.notify("initialized", {})
        client.wait_for(lambda m: m.get("method") == "stateLens/indexing", INDEXING_TIMEOUT_S)
        return client
    except Exception:
        client.terminate()
        raise


def result(client, identifier, item, allow_stale=False):
    response = client.response(identifier)
    method, mutation = client.requests.pop(identifier)
    if "error" in response:
        error = response["error"]
        if not (allow_stale and error["code"] == CONTENT_MODIFIED and mutation < client.mutation):
            item["problems"].append(audit.problem("error", f"{method}: {error}"))
        return None
    return response.get("result")


def check_group(client, paths, rows, baseline, update, summaries):
    texts = {p: p.read_bytes().decode("utf-8") for p in paths}
    params = {p: {"textDocument": {"uri": p.as_uri()}} for p in paths}
    whole = {p: {"start": {"line": 0, "character": 0}, "end": {"line": len(texts[p].split("\n")), "character": 0}} for p in paths}
    for p in paths:
        client.notify("textDocument/didOpen", {"textDocument": {"uri": p.as_uri(), "languageId": EXTENSIONS[p.suffix], "version": OPEN_VERSION, "text": texts[p]}})
    requests = []
    for p in paths:
        for method, extra in (("textDocument/inlayHint", {"range": whole[p]}), ("textDocument/hover", {"position": {"line": 0, "character": 0}}), ("stateLens/ranges", {})):
            requests.append((p, client.begin(method, {**params[p], **extra})))
        client.notify("textDocument/didChange", {"textDocument": {"uri": p.as_uri(), "version": EDITED_VERSION}, "contentChanges": [{"text": texts[p] + "\n"}]})
        requests.append((p, client.begin("textDocument/inlayHint", {**params[p], "range": whole[p]})))
    for p, identifier in requests:
        result(client, identifier, rows[p], allow_stale=True)
    for p in paths:
        client.notify("textDocument/didChange", {"textDocument": {"uri": p.as_uri(), "version": RESTORED_VERSION}, "contentChanges": [{"text": texts[p]}]})
    for p in paths:
        client.wait_for(lambda m: m.get("method") == PUBLISH and m["params"]["uri"] == p.as_uri() and m["params"].get("version") == RESTORED_VERSION)
    stable = {p: client.begin("stateLens/ranges", params[p]) for p in paths}
    for p in paths:
        item = rows[p]
        snapshot = result(client, stable[p], item)
        if snapshot is None:
            item["problems"].append(audit.problem("error", "settled ranges returned no snapshot"))
            continue
        if snapshot["version"] != RESTORED_VERSION or snapshot["sourceHash"] != hashlib.sha256(p.read_bytes()).hexdigest():
            item["problems"].append(audit.problem("error", "settled snapshot version or source hash differs"))
        audit.check_compound(item, snapshot["findings"])
        summary = summarize(snapshot)
        summaries[relative(p)] = summary
        item.update(findings=len(snapshot["findings"]), hinted_functions=summary["hinted_functions"])
        if not update:
            item["problems"] += [audit.problem("baseline-drift", message) for message in compare(baseline.get(relative(p)), summary)]
        lines = texts[p].split("\n")
        triggers = []
        for entry in snapshot["findings"]:
            trigger, function = entry["triggerRange"], entry["functionRange"]
            triggers.append(trigger)
            item["problems"] += [audit.problem("range", message) for message in range_problems("trigger", trigger, lines)]
            if function and not (position_key(function["start"]) <= position_key(trigger["start"]) <= position_key(trigger["end"]) <= position_key(function["end"])):
                item["problems"].append(audit.problem("range", "trigger outside its function"))
            if trigger["start"] != trigger["end"]:
                hover = result(client, client.begin("textDocument/hover", {**params[p], "position": trigger["start"]}), item)
                if not hover or "https://" not in json.dumps(hover):
                    item["problems"].append(audit.problem("error", f"markdown hover lacks guide link at {trigger['start']}"))
        for function in snapshot["functions"]:
            for field in ("range", "body"):
                item["problems"] += [audit.problem("range", message) for message in range_problems(field, function[field], lines)]
        hints = result(client, client.begin("textDocument/inlayHint", {**params[p], "range": whole[p]}), item)
        if hints is not None and len(hints) != summary["hinted_functions"]:
            item["problems"].append(audit.problem("count-mismatch", f"{len(hints)} hints for {summary['hinted_functions']} functions"))
        # A settled ranges response follows the versioned publish on the same stream.
        diagnostics = client.latest_diagnostics.get(p.as_uri(), {})
        if diagnostics.get("version") != RESTORED_VERSION or len(diagnostics.get("diagnostics", [])) != item["findings"]:
            item["problems"].append(audit.problem("count-mismatch", "diagnostics differ from settled snapshot"))
        audit.check_diagnostics(item, diagnostics.get("diagnostics", []))
        audit.finish(item, triggers)
    for p in paths:
        client.notify("textDocument/didClose", params[p])
    for p in paths:
        message = client.wait_for(lambda m: m.get("method") == PUBLISH and m["params"]["uri"] == p.as_uri() and "version" not in m["params"])
        if message["params"]["diagnostics"]:
            rows[p]["problems"].append(audit.problem("count-mismatch", "nonempty diagnostics after close"))
    client.pending.clear()


def run_shard(binary, shard, paths, baseline, update, report):
    client = None
    completed, summaries = [], {}
    for paths_in_group in groups(paths).values():
        rows = {p: audit.row("protocol", relative(p)) for p in paths_in_group}
        started = time.monotonic()
        try:
            if client is None:
                client = initialize(binary, shard)
            client.deadline = time.monotonic() + GROUP_TIMEOUT_S
            check_group(client, paths_in_group, rows, baseline, update, summaries)
        except Exception as error:
            category = "timeout" if isinstance(error, (TimeoutError, subprocess.TimeoutExpired)) else "crash"
            for item in rows.values():
                item["problems"].append(audit.problem(category, str(error)))
            if client:
                for item in rows.values():
                    item["problems"] += client.errors
                client.terminate()
                client = None
        for item in rows.values():
            if client:
                item["problems"] += client.errors
            item["ms"] = round((time.monotonic() - started) * 1000)
            completed.append(audit.finish(item))
        if client:
            client.errors.clear()
        if report:
            audit.write(report, completed)
    if client:
        try:
            if client.shutdown() != 0:
                raise Crash("nonzero shutdown exit")
        except Exception as error:
            for item in completed:
                item["problems"].append(audit.problem("crash", str(error)))
        finally:
            for item in completed:
                item["problems"] += client.errors
                audit.finish(item)
            client.terminate()
    if report:
        audit.write(report, completed)
    return completed, summaries


def run(binary, update, report=None, jobs=None, strict_gaps=False, selected=None):
    files = [p for p in corpus_files() if not selected or relative(p).startswith(selected + "/")]
    if not files:
        raise ValueError("No corpus files selected")
    if selected and update:
        raise ValueError("Cannot update the baseline from a partial corpus")
    baseline = {} if update else json.loads(BASELINE.read_text())["files"]
    shards = collections.defaultdict(list)
    for path in files:
        shards[corpus_shard(path.relative_to(ROOT))].append(path)
    rows, summaries = [], {}
    started = time.monotonic()
    with concurrent.futures.ProcessPoolExecutor(max_workers=min(jobs or os.cpu_count() or 1, len(shards))) as pool:
        futures = {pool.submit(run_shard, binary, shard, paths, baseline, update, report / "protocol-shards" / shard if report else None): (shard, paths) for shard, paths in shards.items()}
        for future in concurrent.futures.as_completed(futures):
            shard, paths = futures[future]
            try:
                results, actual = future.result()
                rows.extend(results)
                summaries.update(actual)
            except Exception as error:
                for path in paths:
                    item = audit.row("protocol", relative(path))
                    item["problems"].append(audit.problem("crash", str(error)))
                    rows.append(audit.finish(item))
            print(f"{shard}: complete ({time.monotonic() - started:.1f}s)", flush=True)
            if report:
                audit.write(report, rows)
    for name in sorted({name for name in baseline if not selected or name.startswith(selected + "/")} - {relative(p) for p in files}):
        item = audit.row("protocol", name)
        item["problems"].append(audit.problem("baseline-drift", "baseline file missing from corpus"))
        rows.append(audit.finish(item))
    if report:
        audit.write(report, rows)
    failures = audit.failed(rows, strict_gaps)
    if update and not failures and len(summaries) == len(files):
        audit.atomic_json(BASELINE, {"schema_version": SCHEMA_VERSION, "purpose": "Regression baseline of editor hints, not ground truth.", "files": dict(sorted(summaries.items()))})
    counts = collections.Counter(r["status"] for r in rows)
    print(f"{dict(counts)} in {time.monotonic() - started:.2f}s")
    for item in rows:
        for issue in item["problems"]:
            print(f"FAIL {item['file']}: {issue['category']}: {issue['message']}")
    return int(failures)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--shard")
    parser.add_argument("--update", action="store_true")
    parser.add_argument("--report", type=Path)
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 1)
    parser.add_argument("--strict-gaps", action="store_true")
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("--jobs must be positive")
    sys.exit(run(args.binary, args.update, args.report, args.jobs, args.strict_gaps, args.shard))


if __name__ == "__main__":
    main()
