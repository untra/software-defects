"""Shared smoke audit serialization, manifest coverage, and report merging."""
import argparse
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCHEMA_VERSION = 1
LAYERS = ("protocol", "editor")
CATEGORIES = {"error", "log-error", "log-warning", "timeout", "range", "count-mismatch", "baseline-drift", "crash"}
FIELDS = {"layer", "file", "family", "defect", "language", "root", "status", "problems", "findings", "hinted_functions", "ms", "bug_lines_hit", "bug_lines_missed"}


def problem(category, message):
    if category not in CATEGORIES:
        raise ValueError(category)
    return {"category": category, "message": str(message)}


def row(layer, file):
    root, family, defect, language, _ = Path(file).parts
    return dict(layer=layer, file=str(file), family=family, defect=defect, language=language,
                root=root, status="pass", problems=[], findings=None, hinted_functions=None,
                ms=0, bug_lines_hit=[], bug_lines_missed=[])


def finish(item, triggers=None):
    if triggers is not None and item["root"] == "examples":
        manifest = ROOT / item["root"] / item["family"] / item["defect"] / "manifest.json"
        examples = json.loads(manifest.read_text())["examples"]
        name = "/".join(Path(item["file"]).parts[-2:])
        lines = next(e["bug_lines"] for e in examples if e["file"] == name)
        covered = set()
        for trigger in triggers:
            start, end = trigger["start"], trigger["end"]
            if start == end:
                continue
            last = end["line"] + (end["character"] > 0)
            covered.update(range(start["line"] + 1, last + 1))
        item["bug_lines_hit"] = sorted(set(lines) & covered)
        item["bug_lines_missed"] = sorted(set(lines) - covered)
    for issue in item["problems"]:
        if issue["category"] in {"baseline-drift", "range", "count-mismatch"}:
            target = f"queries/{item['language']}/{item['family']}/{item['defect']}/signals.scm"
            issue["where_to_look"] = target + (" (production pack absent)" if not (ROOT / target).exists() else "")
        else:
            issue["where_to_look"] = "tooling/state-lens/src/server.rs" + ("; extensions/vscode/src/extension.ts" if item["layer"] == "editor" else "")
    gap = item["root"] == "examples" and (item["hinted_functions"] == 0 or bool(item["bug_lines_missed"]))
    item["status"] = "fail" if item["problems"] else "gap" if gap else "pass"
    return item


def failed(rows, strict=False):
    return any(r["status"] == "fail" or strict and r["status"] == "gap" for r in rows)


def atomic_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f"{path.name}.{os.getpid()}.tmp")
    temporary.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")
    temporary.replace(path)


def validate(item):
    if set(item) != FIELDS or item["layer"] not in LAYERS or item["status"] not in {"pass", "fail", "gap"}:
        raise ValueError(f"Invalid smoke row: {item.get('file')}")
    if any(p["category"] not in CATEGORIES for p in item["problems"]):
        raise ValueError("Invalid problem category")


def write(directory, rows):
    directory = Path(directory)
    rows = sorted(rows, key=lambda r: (r["family"], r["defect"], r["language"], {"fail": 0, "gap": 1, "pass": 2}[r["status"]], r["file"], r["layer"]))
    for item in rows:
        validate(item)
    atomic_json(directory / "smoke-report.json", {"schema_version": SCHEMA_VERSION, "rows": rows})
    counts = {status: sum(r["status"] == status for r in rows) for status in ("pass", "fail", "gap")}
    output = ["# State Lens smoke audit", "", f"{len(rows)} layer/file rows: {counts['pass']} pass, {counts['fail']} fail, {counts['gap']} gap.", "", "Gaps are audit signals, not validated detector accuracy. Manifest bug lines are never analysis input."]
    group = None
    escape = lambda value: str(value).replace("|", "\\|").replace("\n", " ")
    for item in rows:
        key = (item["family"], item["defect"], item["language"])
        if key != group:
            group = key
            output += ["", "## " + " / ".join(key), "", "| Status | Layer | File | Findings / hints | Bug lines hit / missed | ms | Problems / where to look |", "|---|---|---|---|---|---|---|"]
        details = [f"{p['category']}: {p['message']} — {p['where_to_look']}" for p in item["problems"]]
        if item["status"] == "gap":
            target = f"queries/{item['language']}/{item['family']}/{item['defect']}/signals.scm"
            details.append(f"gap: zero hints or missed bug lines — {target}" + (" (production pack absent)" if not (ROOT / target).exists() else ""))
        values = [item["status"], item["layer"], item["file"], f"{item['findings']} / {item['hinted_functions']}", f"{item['bug_lines_hit']} / {item['bug_lines_missed']}", item["ms"], "; ".join(details)]
        output.append("| " + " | ".join(map(escape, values)) + " |")
    target = directory / "smoke-report.md"
    temporary = target.with_suffix(".md.tmp")
    temporary.write_text("\n".join(output) + "\n")
    temporary.replace(target)


def merge(inputs, layers, shard=None):
    from smoke import corpus_files, relative
    indexed = {}
    for source in inputs:
        payload = json.loads(Path(source).read_text())
        if payload["schema_version"] != SCHEMA_VERSION:
            raise ValueError(f"Unsupported report schema: {source}")
        for item in payload["rows"]:
            validate(item)
            key = item["layer"], item["file"]
            if key in indexed:
                raise ValueError(f"Duplicate smoke row: {key}")
            indexed[key] = finish(item)
    for file in map(relative, corpus_files()):
        if shard and not file.startswith(shard + "/"):
            continue
        for layer in layers:
            if (layer, file) not in indexed:
                item = row(layer, file)
                item["problems"].append(problem("crash", "No result: shard did not complete this file"))
                indexed[layer, file] = finish(item)
    return list(indexed.values())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", nargs="*", type=Path)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--layers", nargs="+", choices=LAYERS, default=list(LAYERS))
    parser.add_argument("--shard")
    parser.add_argument("--strict-gaps", action="store_true")
    parser.add_argument("--summary", action="store_true")
    args = parser.parse_args()
    rows = merge(args.inputs, args.layers, args.shard)
    write(args.report, rows)
    if args.summary and os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as summary:
            summary.write((args.report / "smoke-report.md").read_text())
    print(args.report / "smoke-report.md")
    return int(failed(rows, args.strict_gaps))


if __name__ == "__main__":
    raise SystemExit(main())
