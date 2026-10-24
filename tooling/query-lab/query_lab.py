#!/usr/bin/env python3
"""Pinned external Tree-sitter runner and corpus query-development harness."""
from __future__ import annotations

import argparse
import collections
import fcntl
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import selectors
import shutil
import signal
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
import urllib.request

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CACHE = HERE / ".cache"
CLI_VERSION = "0.25.10"
SCHEMA_VERSION = 1
LANGUAGES = {"python": (".py", "0.25.0"), "go": (".go", "0.25.0"), "rust": (".rs", "0.24.0"), "typescript": (".ts", "0.23.2")}
MAX_SOURCE_BYTES = 2 * 1024 * 1024
MAX_OUTPUT_BYTES = 16 * 1024 * 1024
COMMAND_TIMEOUT = 30
BUILD_TIMEOUT = 600
RUNTIME_TIMEOUT = 15
SIDECARS = HERE / "cases"
EXIT_MISMATCH = 1
EXIT_ERROR = 2


class LabError(Exception):
    pass


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_source(path):
    with Path(path).open("rb") as source:
        data = source.read(MAX_SOURCE_BYTES + 1)
    if len(data) > MAX_SOURCE_BYTES:
        raise LabError(f"source exceeds {MAX_SOURCE_BYTES} bytes: {path}")
    data.decode("utf-8")
    return data


def execute(argv, *, cwd=ROOT, timeout=COMMAND_TIMEOUT, env=None):
    """Bound both pipes while draining them, and kill descendants on failure."""
    settings = dict(os.environ, NO_COLOR="1", TERM="dumb")
    settings.update(env or {})
    with subprocess.Popen([str(a) for a in argv], cwd=cwd, env=settings,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          start_new_session=True) as process:
        streams = {"stdout": bytearray(), "stderr": bytearray()}
        deadline = time.monotonic() + timeout
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ, "stdout")
                selector.register(process.stderr, selectors.EVENT_READ, "stderr")
                while selector.get_map():
                    if time.monotonic() >= deadline:
                        raise LabError(f"command timed out after {timeout}s: {argv[0]}")
                    for key, _ in selector.select(min(0.1, max(0, deadline - time.monotonic()))):
                        chunk = os.read(key.fileobj.fileno(), 65536)
                        if not chunk:
                            selector.unregister(key.fileobj)
                            continue
                        streams[key.data].extend(chunk)
                        if sum(map(len, streams.values())) > MAX_OUTPUT_BYTES:
                            raise LabError("command output limit exceeded; results are incomplete")
                process.wait(timeout=max(0.01, deadline - time.monotonic()))
        except BaseException:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
            raise
    return process.returncode, *(bytes(streams[k]).decode("utf-8", errors="replace") for k in ("stdout", "stderr"))


def checked(argv, **kwargs):
    code, out, err = execute(argv, **kwargs)
    if code:
        raise LabError(f"command failed ({code}): {err or out}")
    return out


def package_lock():
    lock = tomllib.loads((ROOT / "tooling/state-lens/Cargo.lock").read_text())
    packages = {p["name"]: p for p in lock["package"] if p["name"] in {f"tree-sitter-{lang}" for lang in LANGUAGES}}
    for lang, (_, version) in LANGUAGES.items():
        package = packages[f"tree-sitter-{lang}"]
        if package["version"] != version:
            raise LabError(f"grammar pin differs from Cargo.lock: {lang}")
    return packages


def cache_key():
    return digest(json.dumps([CLI_VERSION, package_lock(), platform.system(), platform.machine()], sort_keys=True).encode())[:20]


def cli():
    binary = CACHE / "cli/bin/tree-sitter"
    if not binary.is_file():
        raise LabError("Tree-sitter CLI is not installed; run setup")
    version = checked([binary, "--version"]).strip()
    if not re.match(rf"^tree-sitter {re.escape(CLI_VERSION)}(?:\s|$)", version):
        raise LabError(f"wrong CLI version: {version}; run setup")
    return binary


def grammar(lang):
    base = CACHE / cache_key() / lang
    directory = base / "typescript" if lang == "typescript" else base
    if not (directory / "src/parser.c").is_file() or not (base / ".ready").is_file():
        raise LabError(f"grammar {lang} is not installed; run setup")
    if (base / ".ready").read_text().strip() != package_lock()[f"tree-sitter-{lang}"]["checksum"]:
        raise LabError(f"grammar checksum marker differs from lockfile: {lang}; run setup")
    return directory


def config():
    path = CACHE / cache_key() / "config.json"
    if not path.is_file():
        raise LabError("parser configuration missing; run setup")
    return path


def setup_unlocked():
    CACHE.mkdir(parents=True, exist_ok=True)
    try:
        binary = cli()
    except (LabError, OSError):
        checked(["cargo", "install", "tree-sitter-cli", "--version", CLI_VERSION,
                 "--locked", "--force", "--root", CACHE / "cli"], timeout=BUILD_TIMEOUT)
        binary = cli()
    base = CACHE / cache_key()
    base.mkdir(parents=True, exist_ok=True)
    for lang, package in ((lang, package_lock()[f"tree-sitter-{lang}"]) for lang in LANGUAGES):
        destination = base / lang
        if (destination / ".ready").is_file() and (destination / ".ready").read_text().strip() == package["checksum"]:
            continue
        url = f"https://static.crates.io/crates/{package['name']}/{package['name']}-{package['version']}.crate"
        with urllib.request.urlopen(url, timeout=COMMAND_TIMEOUT) as response:
            archive = response.read(MAX_OUTPUT_BYTES + 1)
        if len(archive) > MAX_OUTPUT_BYTES or digest(archive) != package["checksum"]:
            raise LabError(f"package checksum mismatch: {package['name']}")
        with tempfile.TemporaryDirectory(dir=base) as temporary:
            extracted = Path(temporary)
            with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
                tar.extractall(extracted, filter="data")
            package_dir = extracted / f"{package['name']}-{package['version']}"
            if destination.exists():
                shutil.rmtree(destination)
            package_dir.rename(destination)
        parser_dir = destination / "typescript" if lang == "typescript" else destination
        checked([binary, "build"], cwd=parser_dir, timeout=BUILD_TIMEOUT,
                env={"TREE_SITTER_LIBDIR": str(base / "lib")})
        (destination / ".ready").write_text(package["checksum"] + "\n")
    (base / "config.json").write_text(json.dumps({"parser-directories": [str(base)]}))
    with tempfile.TemporaryDirectory(prefix="query-lab-setup-") as temporary:
        for lang, (extension, _) in LANGUAGES.items():
            source = Path(temporary) / ("empty" + extension)
            source.write_text("")
            parse_file(source, lang)
    return doctor()


def setup():
    CACHE.mkdir(parents=True, exist_ok=True)
    with (CACHE / "setup.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        return setup_unlocked()


def doctor():
    binary = cli()
    return {"cli": str(binary), "cli_version": CLI_VERSION, "cache_key": cache_key(),
            "config": str(config()), "grammars": {lang: {"version": version, "path": str(grammar(lang))}
                                                     for lang, (_, version) in LANGUAGES.items()}}


def language(path, explicit=None):
    if explicit:
        return explicit
    for lang, (extension, _) in LANGUAGES.items():
        if Path(path).suffix == extension:
            return lang
    raise LabError(f"unsupported extension: {path}; supply --language")


def native(lang, command, *args):
    return execute([cli(), command, "--config-path", config(), *args], cwd=grammar(lang),
                   env={"TREE_SITTER_LIBDIR": str(CACHE / cache_key() / "lib")})


def metadata(source, lang, query=None):
    value = {"source_hash": digest(source), "language": lang, "grammar_version": LANGUAGES[lang][1],
             "cli_version": CLI_VERSION, "complete": True}
    if query is not None:
        value["query_hash"] = digest(query)
    return value


def offset(source, row, column):
    starts = [0] + [match.end() for match in re.finditer(b"\n", source)]
    if row >= len(starts):
        raise LabError("CLI returned an invalid source row")
    start = starts[row]
    end = starts[row + 1] - 1 if row + 1 < len(starts) else len(source)
    if column > end - start:
        raise LabError("CLI returned an invalid source column")
    result = start + column
    source[:result].decode("utf-8")
    return result


PATTERN = re.compile(r"^  pattern: (\d+)$")
CAPTURE = re.compile(r"^    capture: (?:(\d+) - )?([\w_.-]+), start: \((\d+), (\d+)\), end: \((\d+), (\d+)\)(?:, text: `[^\n]*`)?$")


def decode_matches(output, source, source_path):
    lines = output.split("\n")
    if lines and lines[0] == str(source_path):
        lines.pop(0)
    else:
        raise LabError("unexpected CLI output header")
    matches = []
    for line in lines:
        if not line:
            continue
        pattern = PATTERN.fullmatch(line)
        if pattern:
            matches.append({"pattern": int(pattern[1]), "captures": []})
            continue
        capture = CAPTURE.fullmatch(line)
        if not capture or not matches:
            raise LabError(f"unexpected CLI output; results are incomplete: {line[:200]}")
        start = offset(source, int(capture[3]), int(capture[4]))
        end = offset(source, int(capture[5]), int(capture[6]))
        if end < start:
            raise LabError("CLI returned a reversed capture range")
        matches[-1]["captures"].append({"name": capture[2], "range": [start, end],
                                        "text": source[start:end].decode("utf-8")})
    return matches


def parse_file(path, lang=None, lines=None):
    path = Path(path).resolve()
    lang = language(path, lang)
    source = read_source(path)
    with tempfile.TemporaryDirectory(prefix="query-lab-parse-") as temporary:
        snapshot = Path(temporary) / ("source" + LANGUAGES[lang][0])
        snapshot.write_bytes(source)
        code, out, err = native(lang, "parse", snapshot)
        out = out.replace(str(snapshot), str(path))
    if code not in (0, 1) or err:
        raise LabError(err or out)
    tree = out
    if lines:
        start, end = map(int, lines.split(":"))
        if start < 1 or end < start:
            raise LabError("--lines must be an inclusive one-indexed START:END")
        selected = []
        for line in out.splitlines():
            position = re.search(r"\[(\d+), \d+\] - \[(\d+), \d+\]", line)
            if position and int(position[1]) <= end - 1 and int(position[2]) >= start - 1:
                selected.append(line)
        tree = "\n".join(selected)
    return dict(metadata(source, lang), file=str(path), tree=tree, parse_errors=code != 0)


def run_query(query, source_path, lang=None):
    query, source_path = Path(query).resolve(), Path(source_path).resolve()
    lang = language(source_path, lang)
    source, query_data = read_source(source_path), read_source(query)
    validate_predicates(query_data.decode("utf-8"))
    with tempfile.TemporaryDirectory(prefix="query-lab-run-") as temporary:
        snapshot = Path(temporary) / ("source" + LANGUAGES[lang][0])
        query_snapshot = Path(temporary) / "query.scm"
        snapshot.write_bytes(source)
        query_snapshot.write_bytes(query_data)
        parsed = parse_file(snapshot, lang)
        code, out, err = native(lang, "query", query_snapshot, snapshot)
        matches = decode_matches(out, source, snapshot) if not code and not err else []
    if code or err:
        raise LabError(f"{query}: {err or out}")
    return dict(metadata(source, lang, query_data), file=str(source_path), query=str(query),
                parse_errors=parsed["parse_errors"], matches=matches)


SUPPORTED_PREDICATES = frozenset({"eq?", "not-eq?", "any-eq?", "any-not-eq?", "match?",
                                  "not-match?", "any-match?", "any-not-match?", "any-of?", "not-any-of?"})


def validate_predicates(query):
    without_literals = re.sub(r'"(?:\\.|[^"\\])*"|;[^\n]*', "", query)
    for predicate in re.findall(r"\(\s*[#.]([\w?!-]+)", without_literals):
        if predicate not in SUPPORTED_PREDICATES:
            raise LabError(f"unsupported predicate/directive: {predicate}; it cannot be evaluated by this runner")


def check_query(query, lang):
    with tempfile.TemporaryDirectory() as temporary:
        source = Path(temporary) / ("empty" + LANGUAGES[lang][0])
        source.write_text("")
        result = run_query(query, source, lang)
    return {key: value for key, value in result.items() if key not in {"file", "source_hash", "matches"}}


def corpus(*, include_unsupported=False):
    result = []
    for path in sorted((ROOT / "examples").glob("*/*/manifest.json")):
        manifest = json.loads(path.read_text())
        for example in manifest["examples"]:
            supported = example["language"] in LANGUAGES
            if not supported and not include_unsupported:
                continue
            relative = (path.parent / example["file"]).relative_to(ROOT / "examples")
            case_id = relative.with_suffix("").as_posix()
            sidecar = SIDECARS / relative.with_suffix(".json")
            case = {"id": case_id, "family": manifest["family"], "defect": manifest["defect"],
                    "language": example["language"], "original": str(Path("examples") / relative),
                    "fixed": str(Path("examples-fixed") / relative), "sidecar": str(sidecar.relative_to(ROOT)),
                    "status": "missing" if supported else "unsupported-language", "bug_lines": example["bug_lines"]}
            if sidecar.is_file():
                case["expectations"] = json.loads(sidecar.read_text())
                case["status"] = case["expectations"]["status"]
            result.append(case)
    return result


def select(args):
    cases = corpus()
    for key in ("language", "family", "defect"):
        value = getattr(args, key, None)
        if value:
            cases = [case for case in cases if case[key] == value]
    if getattr(args, "case", None):
        cases = [case for case in cases if case["id"] == args.case]
    if not cases:
        raise LabError("no corpus cases match the selection")
    return cases


def capture_counts(matches):
    return collections.Counter((capture["name"], *capture["range"]) for match in matches for capture in match["captures"])


def expanded(counter):
    return [{"name": name, "range": [start, end]} for (name, start, end), count in sorted(counter.items()) for _ in range(count)]


def test_case(case):
    expectation = case.get("expectations")
    if not expectation or expectation.get("schema_version") != SCHEMA_VERSION:
        raise LabError(f"missing or invalid sidecar: {case['id']}")
    result = {"id": case["id"], "status": case["status"], "variants": {}, "passed": True,
              "capture_checks": 0, "validation": "parse-only"}
    for variant in ("original", "fixed"):
        path = ROOT / case[variant]
        data = read_source(path)
        expected = expectation[variant]
        if digest(data) != expected["source_hash"]:
            raise LabError(f"stale expectations: {path}")
        parsed = parse_file(path, case["language"])
        if parsed["parse_errors"]:
            raise LabError(f"parse errors: {path}")
        checks = []
        for spec in expected["queries"]:
            report = run_query(ROOT / spec["query"], path, case["language"])
            actual = capture_counts(report["matches"])
            wanted = collections.Counter((capture["name"], *capture["range"]) for capture in spec["captures"])
            check = {"query": spec["query"], "missing": expanded(wanted - actual), "unexpected": expanded(actual - wanted),
                     "actual": expanded(actual), "query_hash": report["query_hash"]}
            result["passed"] &= not check["missing"] and not check["unexpected"]
            checks.append(check)
            result["capture_checks"] += 1
        result["variants"][variant] = checks
    if result["capture_checks"]:
        result["validation"] = "captures"
    return result


def coverage(cases):
    cells = collections.defaultdict(collections.Counter)
    for case in cases:
        cell = cells[(case["family"], case["defect"], case["language"])]
        cell["examples"] += 1
        cell[case["status"]] += 1
        cell["fixed_files"] += (ROOT / case["fixed"]).is_file()
        cell["capture_tested"] += any(case.get("expectations", {}).get(v, {}).get("queries", []) for v in ("original", "fixed"))
        for variant in ("original", "fixed"):
            cell[f"{variant}_with_candidates"] += any(spec["captures"] for spec in case.get("expectations", {}).get(variant, {}).get("queries", []))
    return {"total": len(cases), "cells": [dict(family=f, defect=d, language=l, **counts)
                                            for (f, d, l), counts in sorted(cells.items())]}


def verify_fix(case):
    expectation = case.get("expectations", {})
    runtime = expectation.get("runtime", {})
    if runtime.get("status") != "supported":
        return {"id": case["id"], "status": "unsupported", "reason": runtime.get("reason", "no reviewed runtime check")}
    path = ROOT / case["fixed"]
    source = read_source(path)
    if digest(source) != expectation["fixed"]["source_hash"]:
        raise LabError(f"stale fixed source: {path}")
    with tempfile.TemporaryDirectory(prefix="query-lab-") as temporary:
        work = Path(temporary)
        copied = work / ("main" + LANGUAGES[case["language"]][0])
        copied.write_bytes(source)
        if case["language"] == "go":
            shutil.copyfile(ROOT / "examples/go.mod", work / "go.mod")
            checked(["go", "build", "-o", work / "program", copied], cwd=work, timeout=BUILD_TIMEOUT)
            command = [work / "program"]
        elif case["language"] == "rust":
            checked(["rustc", "--edition", "2021", copied, "-o", work / "program"], cwd=work, timeout=BUILD_TIMEOUT)
            command = [work / "program"]
        elif case["language"] == "python":
            command = [sys.executable, copied]
        else:
            command = ["node", copied]
        code, out, err = execute([sys.executable, HERE / "runtime_child.py", *command], cwd=work,
                                 timeout=min(runtime.get("timeout_seconds", RUNTIME_TIMEOUT), COMMAND_TIMEOUT),
                                 env={"TMPDIR": str(work), "TMP": str(work), "TEMP": str(work)})
    passed = code == runtime.get("exit_code", 0)
    if "stdout" in runtime:
        passed &= out == runtime["stdout"]
    for fragment in runtime.get("stdout_contains", []):
        passed &= fragment in out
    for fragment in runtime.get("stdout_excludes", []):
        passed &= fragment not in out
    return {"id": case["id"], "source_hash": digest(source), "status": "passed" if passed else "failed",
            "exit_code": code, "stdout": out, "stderr": err}


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("json", "text"), default="json")
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("setup", "doctor"):
        commands.add_parser(name)
    nodes = commands.add_parser("nodes")
    nodes.add_argument("--language", choices=LANGUAGES, required=True)
    nodes.add_argument("--kind")
    parse = commands.add_parser("parse")
    parse.add_argument("file", type=Path)
    parse.add_argument("--language", choices=LANGUAGES)
    parse.add_argument("--lines")
    check = commands.add_parser("check")
    check.add_argument("--query", type=Path, required=True)
    check.add_argument("--language", choices=LANGUAGES, required=True)
    run = commands.add_parser("run")
    run.add_argument("--query", type=Path, required=True)
    run.add_argument("--source", type=Path, required=True)
    run.add_argument("--language", choices=LANGUAGES)
    for name in ("list", "test", "compare", "coverage", "verify-fixes"):
        command = commands.add_parser(name)
        command.add_argument("--language", choices=LANGUAGES)
        command.add_argument("--family")
        command.add_argument("--defect")
        command.add_argument("--case", required=name == "compare")
    for command in commands.choices.values():
        command.add_argument("--format", choices=("json", "text"), default=argparse.SUPPRESS)
    return parser.parse_args()


def dispatch(args):
    if args.command == "setup":
        return setup(), 0
    if args.command == "doctor":
        return doctor(), 0
    if args.command == "nodes":
        nodes = json.loads((grammar(args.language) / "src/node-types.json").read_text())
        return {"nodes": [node for node in nodes if not args.kind or node["type"] == args.kind]}, 0
    if args.command == "parse":
        result = parse_file(args.file, args.language, args.lines)
        return result, EXIT_ERROR if result["parse_errors"] else 0
    if args.command == "check":
        return check_query(args.query, args.language), 0
    if args.command == "run":
        result = run_query(args.query, args.source, args.language)
        return result, EXIT_ERROR if result["parse_errors"] else 0
    cases = select(args)
    if args.command == "list":
        rules = json.loads((HERE / "queries/rules.json").read_text())["rules"]
        selected = {(c["language"], c["family"], c["defect"]) for c in cases}
        return {"cases": [{k: v for k, v in c.items() if k != "expectations"} for c in cases],
                "rules": [r for r in rules if (r["language"], r["family"], r["defect"]) in selected]}, 0
    if args.command == "coverage":
        return coverage(cases), 0
    if args.command == "verify-fixes":
        results = []
        for case in cases:
            try:
                results.append(verify_fix(case))
            except (LabError, OSError) as error:
                results.append({"id": case["id"], "status": "error", "error": str(error)})
        code = EXIT_ERROR if any(r["status"] == "error" for r in results) else EXIT_MISMATCH if any(r["status"] == "failed" for r in results) else 0
        return {"cases": results, "complete": code != EXIT_ERROR}, code
    results = []
    for case in cases:
        try:
            results.append(test_case(case))
        except (LabError, OSError, KeyError, ValueError) as error:
            results.append({"id": case["id"], "passed": False, "error": str(error)})
    if args.command == "compare":
        case = cases[0]
        results[0]["interpretation"] = case["expectations"]["rationale"]
        results[0]["expected"] = {v: case["expectations"][v]["queries"] for v in ("original", "fixed")}
    code = EXIT_ERROR if any("error" in r for r in results) else EXIT_MISMATCH if any(not r["passed"] for r in results) else 0
    return {"cases": results, "complete": code != EXIT_ERROR}, code


def text_report(report):
    if "error" in report:
        return report["error"]
    if "tree" in report:
        return report["tree"]
    if "matches" in report:
        lines = [f"{report['file']} ({report['language']}, grammar {report['grammar_version']})"]
        for match in report["matches"]:
            for capture in match["captures"]:
                lines.append(f"pattern {match['pattern']} @{capture['name']} {capture['range']}: {capture['text']!r}")
        return "\n".join(lines + [f"{len(report['matches'])} matches; parse errors: {report['parse_errors']}"])
    if "cases" in report:
        lines = []
        for case in report["cases"]:
            status = ("passed" if case["passed"] else "failed") if "passed" in case else case["status"]
            lines.append(f"{status}: {case['id']}" + (f" ({case['validation']})" if "validation" in case else ""))
            if "error" in case:
                lines.append(case["error"])
            for variant, checks in case.get("variants", {}).items():
                for check in checks:
                    if check["missing"] or check["unexpected"]:
                        lines.append(f"  {variant}: missing={check['missing']}, unexpected={check['unexpected']}")
            if "interpretation" in case:
                lines.append(case["interpretation"])
        return "\n".join(lines + [f"{len(report['cases'])} cases; complete: {report['complete']}; unsupported: {len(report.get('unsupported_cases', []))}"])
    return json.dumps(report, indent=2, ensure_ascii=False)


def main():
    args = arguments()
    try:
        result, code = dispatch(args)
        envelope = {"schema_version": SCHEMA_VERSION, "command": args.command, "complete": True, **result}
        if args.command in {"list", "coverage", "test", "compare", "verify-fixes"}:
            unsupported = [case for case in corpus(include_unsupported=True) if case["language"] not in LANGUAGES]
            for key in ("language", "family", "defect"):
                if getattr(args, key, None):
                    unsupported = [case for case in unsupported if case[key] == getattr(args, key)]
            if getattr(args, "case", None):
                unsupported = [case for case in unsupported if case["id"] == args.case]
            envelope["unsupported_cases"] = [{"id": c["id"], "language": c["language"], "reason": "grammar not supported"} for c in unsupported]
            envelope["supported_languages"] = list(LANGUAGES)
    except (LabError, OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        envelope, code = {"schema_version": SCHEMA_VERSION, "command": args.command, "complete": False,
                          "error": str(error)}, EXIT_ERROR
    if args.format == "text":
        print(text_report(envelope))
    else:
        print(json.dumps(envelope, indent=2, ensure_ascii=False))
    return code


if __name__ == "__main__":
    sys.exit(main())
