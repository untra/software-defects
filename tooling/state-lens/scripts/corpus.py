#!/usr/bin/env python3
"""Inventory the authored corpus, or export isolated source-only blind cases."""
import argparse
import hashlib
import json
import random
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXAMPLES = ROOT / "examples"
FAMILIES = ("all", "mutated-state", "shared-state", "unstructured-state", "compound-defects")
RISK_ASSESSMENTS = {"candidate", "misuse"}
DEFAULT_BINARY = ROOT / "tooling/state-lens/target/debug/state-lens"
EXTENSIONS = {"go": ".go", "rust": ".rs", "javascript": ".js", "typescript": ".ts", "python": ".py"}
ANALYSIS_RESOLUTIONS = {"binding-analysis", "modeled-api", "resolved-call"}
OBJECT_IDENTITY = "object-identity"


def cases(family="all"):
    base = EXAMPLES if family == "all" else EXAMPLES / family
    for manifest_path in sorted(base.rglob("manifest.json")):
        manifest = json.loads(manifest_path.read_text())
        for example in manifest["examples"]:
            if example["language"] not in EXTENSIONS:
                continue
            source = manifest_path.parent / example["file"]
            yield manifest, example, source


def overlaps_bug_lines(finding, document, source, bug_lines):
    lines = source.splitlines(keepends=True)
    ranges = []
    offset = 0
    for number, line in enumerate(lines, 1):
        end = offset + len(line.encode("utf-8"))
        if number in bug_lines:
            ranges.append((offset, end))
        offset = end
    spans = [finding["trigger"]] + [e["range"] for e in finding.get("evidence", []) if e["uri"] == document["uri"]]
    return any(span["start"] < end and start < span["end"] for span in spans for start, end in ranges)


def summarize(rows):
    groups = {}
    for row in rows:
        for language in ("all", row["language"]):
            key = (row["family"], row["defect"], language)
            group = groups.setdefault(key, [])
            group.append(row)
    result = []
    for (family, defect, language), group in sorted(groups.items()):
        fixed = [r for r in group if r["fixed"]["status"] == "scanned"]
        recalled = sum(r["bug_line_recalled"] for r in group)
        flagged = sum(r["fixed"]["risk_flagged"] for r in fixed)
        result.append(dict(family=family, defect=defect, language=language,
                           examples=len(group), recalled=recalled, recall=recalled / len(group),
                           fixed_scanned=len(fixed), fixed_missing=sum(r["fixed"]["status"] == "missing" for r in group),
                           fixed_risk_flagged=flagged, fixed_risk_rate=flagged / len(fixed) if fixed else None,
                           fixed_marker_flagged=sum(r["fixed"]["marker_flagged"] for r in fixed),
                           parse_error_files=sum(bool(r["parse_errors"]) for r in group),
                           fixed_parse_error_files=sum(bool(r["fixed"]["parse_errors"]) for r in fixed)))
    return result


def inventory(binary, family="all"):
    examples = list(cases(family))
    paths = []
    for _, _, source in examples:
        paths.append(source)
        fixed = ROOT / "examples-fixed" / source.relative_to(EXAMPLES)
        if fixed.exists():
            paths.append(fixed)
    source_bytes = {path: path.read_bytes() for path in paths}
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    output = subprocess.check_output([str(binary), "scan-isolated"], input=json.dumps([str(p) for p in paths]).encode())
    if hashlib.sha256(binary.read_bytes()).hexdigest() != binary_hash:
        raise RuntimeError("Detector binary changed during inventory")
    reports = dict(zip(paths, (json.loads(line) for line in output.splitlines()), strict=True))
    for path, report in reports.items():
        expected = hashlib.sha256(source_bytes[path]).hexdigest()
        if report["documents"][0]["source_hash"] != expected or path.read_bytes() != source_bytes[path]:
            raise RuntimeError(f"Source changed during inventory: {path}")
    rows = []
    for manifest, example, source in examples:
        report = reports[source]
        document = report["documents"][0]
        signals = [f for f in document["findings"] if f["defect"] == manifest["defect"]]
        risks = [f for f in signals if f.get("assessment") in RISK_ASSESSMENTS]
        fixed_path = ROOT / "examples-fixed" / source.relative_to(EXAMPLES)
        fixed = {"status": "missing"}
        if fixed_path.exists():
            fixed_doc = reports[fixed_path]["documents"][0]
            fixed_signals = [f for f in fixed_doc["findings"] if f["defect"] == manifest["defect"]]
            fixed = dict(status="scanned", file=str(fixed_path.relative_to(ROOT)),
                         source_hash=fixed_doc["source_hash"], parse_errors=fixed_doc["parse_errors"],
                         marker_flagged=bool(fixed_signals),
                         risk_flagged=any(f.get("assessment") in RISK_ASSESSMENTS for f in fixed_signals))
        status = "unsupported"
        rationale = "No category-specific signal currently matches; requires additional queries or semantic/domain analysis."
        if signals:
            status = "query-plus-analysis" if any(f["resolution"] in ANALYSIS_RESOLUTIONS for f in signals) else "query-only"
            rationale = "Observed mechanism candidates only; no validated bug-pattern rule exists for this scenario."
        if any(f["resolution"] == OBJECT_IDENTITY for f in signals):
            status = "semantic-relationship"
            rationale = "Object-identity relationship (sharing origin, write, observed read) established from the shared graph; isolation intent is not known, so bug detection remains unsupported."
        rows.append({
            "file": str(source.relative_to(ROOT)),
            "family": manifest["family"], "defect": manifest["defect"], "language": example["language"],
            "source_hash": document["source_hash"],
            "bug_lines": example["bug_lines"],
            "bug_line_recalled": any(overlaps_bug_lines(f, document, source_bytes[source].decode("utf-8"), example["bug_lines"]) for f in risks),
            "marker_bug_line_overlap": any(overlaps_bug_lines(f, document, source_bytes[source].decode("utf-8"), example["bug_lines"]) for f in signals),
            "risk_count": len(risks),
            "fixed": fixed,
            "susceptibility_status": status,
            "observed_rule_ids": sorted({f["rule_id"] for f in signals}),
            "trigger_count": len(signals),
            "bug_detection": "unsupported",
            "mechanisms": sorted({f["mechanism"] for f in signals if f.get("mechanism")}),
            "rationale": rationale,
            "parse_errors": document["parse_errors"],
            "split": "development",
        })
    return {"schema_version": 2, "purpose": "Authored-corpus risk recall and fixed-file flagging; not population accuracy or proof of a bug.", "family": family, "binary_hash": binary_hash, "summary": summarize(rows), "cases": rows}


def blind(destination, key_file, family="all"):
    destination = destination.resolve()
    key_file = key_file.resolve()
    if destination == key_file or destination in key_file.parents:
        raise ValueError("Keep the private mapping outside the detector's source directory")
    if destination.exists() or key_file.exists():
        raise ValueError("Output directory and mapping must not already exist")
    examples = list(cases(family))
    random.SystemRandom().shuffle(examples)
    destination.mkdir(parents=True)
    mapping = []
    for index, (manifest, example, source) in enumerate(examples):
        identifier = f"case_{index:04d}"
        directory = destination / identifier
        directory.mkdir()
        file = directory / ("main" + EXTENSIONS[example["language"]])
        shutil.copyfile(source, file)
        if example["language"] == "go":
            shutil.copyfile(EXAMPLES / "go.mod", directory / "go.mod")
        mapping.append({"case": identifier, "file": str(file.relative_to(destination)), "original": str(source.relative_to(ROOT)), "family": manifest["family"], "defect": manifest["defect"], "source_hash": hashlib.sha256(source.read_bytes()).hexdigest(), "ground_truth": example})
    key_file.parent.mkdir(parents=True, exist_ok=True)
    key_file.write_text(json.dumps({"schema_version": 1, "cases": mapping}, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    coverage = commands.add_parser("inventory")
    coverage.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    export = commands.add_parser("blind")
    export.add_argument("destination", type=Path)
    export.add_argument("--key-file", type=Path, required=True)
    for command in (coverage, export):
        command.add_argument("--family", choices=FAMILIES, default="all")
    args = parser.parse_args()
    if args.command == "inventory":
        print(json.dumps(inventory(args.binary.resolve(), args.family), indent=2))
    else:
        blind(args.destination, args.key_file, args.family)


if __name__ == "__main__":
    main()
