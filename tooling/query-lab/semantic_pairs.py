"""Reviewed contracts are independent of source-only semantic fact extraction."""
from __future__ import annotations

import copy
import difflib
import json
from pathlib import Path
import tempfile

SCHEMA_VERSION = 1
SOURCE_SUFFIXES = frozenset({".go", ".rs", ".js", ".ts", ".py", ".rb", ".java", ".kt", ".c", ".cpp", ".sh", ".sol", ".tf", ".hcl"})


def extract(path, lab, analyzer):
    code, out, err = lab.execute([analyzer.resolve(), "semantic-facts", path])
    if code or err:
        raise ValueError(err or "semantic extraction failed")
    return json.loads(out)


def witness(facts, spec):
    events = facts["events"]
    if facts["unsupported"]:
        return {"status": "unsupported", "reasons": facts["unsupported"]}
    if spec["pattern"] == "binding-consumer-mismatch":
        contract = spec["contract"]
        selected = []
        for name in contract["bindings"]:
            assignments = [e for e in events if e["kind"] == "assignment" and e["function"] == contract["function"] and e["name"] == name and e["previous_binding"] is not None and e.get("producer") == contract["producer"]]
            reads = [e for e in events if e["kind"] == "return_read" and e["function"] == contract["function"] and e["name"] == name]
            if len(assignments) != 1 or len(reads) != 1:
                return {"status": "ambiguous", "reason": "assignment/consumer correspondence is not unique"}
            selected.append({"origin": assignments[0], "affected_use": reads[0], "disconnected": assignments[0]["binding"] != reads[0]["binding"]})
        return {"status": "supported", "failure": any(e["disconnected"] for e in selected), "paths": selected,
                "invalidation": {"status": "not-established", "reason": "producer-to-consumer binding connection was never established"}}
    if spec["pattern"] == "shared-nested-object":
        contract = spec["contract"]
        writes = [e for e in events if e["kind"] == "write" and e["path"] == contract["write"]]
        reads = [e for e in events if e["kind"] == "read" and e["path"] == contract["read"]]
        if len(writes) != 1 or len(reads) != 1:
            return {"status": "ambiguous", "reason": "write/read correspondence is not unique"}
        write, read = writes[0], reads[0]
        same = write["object"] == read["object"] and write["property"] == read["property"]
        later = write["range"]["start"] < read["range"]["start"]
        allocations = [e for e in events if e["kind"] == "allocation" and e["object"] == write["object"]]
        return {"status": "supported", "failure": same and later, "origin": allocations,
                "invalidation": write if same and later else None, "affected_use": read,
                "evidence_path": [*allocations, write, read], "same_object": same}
    return {"status": "unsupported", "reason": "unknown semantic pattern"}


def compare(case, lab, analyzer):
    spec = case.get("expectations", {}).get("semantic")
    if not spec:
        return {"id": case["id"], "status": "unsupported", "reason": "no reviewed semantic specification"}
    if spec.get("schema_version") != SCHEMA_VERSION or spec.get("review_status") != "reviewed":
        return {"id": case["id"], "status": "unsupported", "reason": "specification requires review"}
    variants = {}
    for variant in ("original", "fixed"):
        path = lab.ROOT / case[variant]
        if lab.digest(lab.read_source(path)) != spec["source_hashes"][variant]:
            return {"id": case["id"], "status": "stale", "variant": variant}
        facts = extract(path, lab, analyzer)
        variants[variant] = {"facts": facts, "witness": witness(facts, spec)}
    before, after = [variants[v]["witness"] for v in ("original", "fixed")]
    status = next((v["status"] for v in (before, after) if v["status"] != "supported"), None)
    if status is None:
        status = "validated" if before["failure"] and not after["failure"] else "failed"
    correspondence = []
    if status == "validated":
        if spec["pattern"] == "binding-consumer-mismatch":
            correspondence = [{"role":"producer assignment", "original":a["origin"]["statement"], "fixed":b["origin"]["statement"], "match":"reviewed function, producer and target binding role"} for a,b in zip(before["paths"],after["paths"])]
        else:
            correspondence = [{"role":"affected use", "original":before["affected_use"]["range"], "fixed":after["affected_use"]["range"], "match":"reviewed access path"}]
    return {"id": case["id"], "status": status, "specification": spec, "variants": variants, "correspondence":correspondence}


def preview(case, lab, analyzer, run_behavior=False):
    comparison = compare(case, lab, analyzer)
    if comparison["status"] != "validated":
        return comparison
    spec = comparison["specification"]
    source = lab.read_source(lab.ROOT / case["original"])
    before = comparison["variants"]["original"]
    if spec["pattern"] == "binding-consumer-mismatch":
        statements = {(p["origin"]["statement"]["start"], p["origin"]["statement"]["end"]) for p in before["witness"]["paths"]}
        if len(statements) != 1:
            raise ValueError("repair requires one unambiguous assignment")
        start, end = next(iter(statements))
        targets = [e for e in before["facts"]["events"] if e["kind"] == "assignment" and e["statement"] == {"start":start,"end":end}]
        if not targets or any(e["previous_binding"] is None for e in targets):
            raise ValueError("every replacement target must have an existing binding")
        statement = source[start:end]
        if statement.count(b":=") != 1:
            raise ValueError("repair requires a short declaration")
        replacement = statement.replace(b":=", b"=", 1)
    else:
        target, field, _ = spec["contract"]["write"].split(".")
        origin, source_field, _ = spec["contract"]["read"].split(".")
        candidates = [e for e in before["facts"]["events"] if e["kind"] == "object_binding" and e["name"] == target]
        if len(candidates) != 1:
            raise ValueError("repair target is ambiguous")
        start, end = candidates[0]["range"]["start"], candidates[0]["range"]["end"]
        expression = source[start:end].decode()
        if not expression.startswith("{") or not expression.endswith("}"):
            raise ValueError("repair target must be an object literal")
        replacement = (expression[:-1].rstrip().rstrip(",") + f", {field}: {{ ...{origin}.{source_field} }}" + " }").encode()
    candidate = source[:start] + replacement + source[end:]
    with tempfile.TemporaryDirectory(prefix="semantic-repair-") as directory:
        path = Path(directory) / ("main" + lab.LANGUAGES[case["language"]][0])
        path.write_bytes(candidate)
        after = extract(path, lab, analyzer)
        result = witness(after, spec)
        if result["status"] != "supported" or result["failure"]:
            raise ValueError("semantic repair postcondition failed")
        runtime_case = copy.deepcopy(case)
        runtime_case["fixed"] = str(path)
        runtime_case["expectations"]["fixed"]["source_hash"] = lab.digest(candidate)
        behavior = lab.verify_fix(runtime_case) if run_behavior else {"status": "not-run"}
        baseline = {"status": "not-run"}
        regressions = []
        if run_behavior:
            path.write_bytes(source)
            runtime_case["expectations"]["fixed"]["source_hash"] = lab.digest(source)
            baseline = lab.verify_fix(runtime_case)
            for variant in spec.get("regression_cases", []):
                old, new = variant["replace"].encode(), variant["with"].encode()
                if candidate.count(old) != 1:
                    raise ValueError("regression variant target is ambiguous")
                data = candidate.replace(old, new, 1)
                path.write_bytes(data)
                runtime_case["expectations"]["fixed"]["source_hash"] = lab.digest(data)
                runtime_case["expectations"]["runtime"] = variant["runtime"]
                regressions.append(lab.verify_fix(runtime_case))
            if baseline["status"] != "failed" or any(r["status"] != "passed" for r in regressions):
                behavior = {"status": "failed", "reason": "baseline or regression contract failed"}
    patch = "".join(difflib.unified_diff(source.decode().splitlines(True), candidate.decode().splitlines(True), fromfile=case["original"], tofile="candidate"))
    return {"id":case["id"], "status":"preview", "mode":"semantic", "patch":patch, "postcondition":result,"behavior":behavior,"baseline_behavior":baseline,"regressions":regressions,
            "assumptions":spec["assumptions"]}


def inventory(lab):
    rows = []
    originals = set()
    for manifest in (lab.ROOT / "examples").rglob("manifest.json"):
        for example in json.loads(manifest.read_text()).get("examples", []):
            path = manifest.parent / example["file"]
            originals.add(path.relative_to(lab.ROOT / "examples"))
    fixed = {p.relative_to(lab.ROOT / "examples-fixed") for p in (lab.ROOT / "examples-fixed").rglob("*") if p.is_file() and p.suffix in SOURCE_SUFFIXES}
    for relative in sorted(originals | fixed):
        status = "unsupported"
        sidecar = lab.SIDECARS / relative.with_suffix(".json")
        spec = json.loads(sidecar.read_text()).get("semantic") if sidecar.exists() else None
        if relative not in originals or relative not in fixed:
            status = "unpaired"
        elif spec:
            status = "reviewed"
            for variant, root in (("original", "examples"), ("fixed", "examples-fixed")):
                if lab.digest(lab.read_source(lab.ROOT / root / relative)) != spec["source_hashes"][variant]:
                    status = "stale"
        rows.append({"path": str(relative), "status": status, "paired": relative in originals and relative in fixed})
    return {"schema_version":SCHEMA_VERSION, "cases":rows}
