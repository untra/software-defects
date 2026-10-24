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
CURRENT_FAMILY = "mutated-state"
DEFAULT_BINARY = ROOT / "tooling/state-lens/target/debug/state-lens"
EXTENSIONS = {"go": ".go", "rust": ".rs", "typescript": ".ts"}


def cases():
    for manifest_path in sorted(EXAMPLES.glob(f"{CURRENT_FAMILY}/*/manifest.json")):
        manifest = json.loads(manifest_path.read_text())
        for example in manifest["examples"]:
            if example["language"] not in EXTENSIONS:
                continue
            source = manifest_path.parent / example["file"]
            yield manifest, example, source


def inventory(binary):
    rows = []
    for manifest, example, source in cases():
        report = json.loads(subprocess.check_output([str(binary), "scan", str(source)]))
        document = report["documents"][0]
        signals = [f for f in document["findings"] if f["defect"] == manifest["defect"]]
        status = "unsupported"
        if signals:
            status = "query-plus-analysis" if any(f["resolution"] in {"binding-analysis", "modeled-api", "resolved-call"} for f in signals) else "query-only"
        rows.append({
            "file": str(source.relative_to(ROOT)),
            "family": manifest["family"], "defect": manifest["defect"], "language": example["language"],
            "source_hash": document["source_hash"],
            "susceptibility_status": status,
            "observed_rule_ids": sorted({f["rule_id"] for f in signals}),
            "trigger_count": len(signals),
            "bug_detection": "unsupported",
            "rationale": "Observed mechanism candidates only; no validated bug-pattern rule exists for this scenario." if signals else "No category-specific signal currently matches; requires additional queries or semantic/domain analysis.",
            "parse_errors": document["parse_errors"],
            "split": "development",
        })
    return {"schema_version": 1, "purpose": "Coverage accounting, not ground truth or detector accuracy.", "cases": rows}


def blind(destination, key_file):
    destination = destination.resolve()
    key_file = key_file.resolve()
    if destination == key_file or destination in key_file.parents:
        raise ValueError("Keep the private mapping outside the detector's source directory")
    if destination.exists() or key_file.exists():
        raise ValueError("Output directory and mapping must not already exist")
    examples = list(cases())
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
    args = parser.parse_args()
    if args.command == "inventory":
        print(json.dumps(inventory(args.binary.resolve()), indent=2))
    else:
        blind(args.destination, args.key_file)


if __name__ == "__main__":
    main()
