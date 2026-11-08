"""Review-only repair previews for source-hashed repository fixtures."""
from __future__ import annotations

import difflib
import json
from pathlib import Path
import tempfile

SUPPORTED_RECIPES = frozenset({"denominator-guard", "independent-default"})


def apply_recipe(source: bytes, recipe: dict, digest) -> bytes:
    if recipe["kind"] not in SUPPORTED_RECIPES:
        raise ValueError("unsupported repair recipe")
    if digest(source) != recipe["source_hash"]:
        raise ValueError("stale repair source")
    edits = sorted(recipe["edits"], key=lambda edit: edit["range"])
    end = 0
    output = bytearray()
    for edit in edits:
        start, stop = edit["range"]
        if not 0 <= end <= start <= stop <= len(source):
            raise ValueError("overlapping or invalid repair range")
        source[:start].decode("utf-8")
        source[:stop].decode("utf-8")
        if source[start:stop].decode("utf-8") != edit["expected"]:
            raise ValueError("repair precondition mismatch")
        output.extend(source[end:start])
        output.extend(edit["replacement"].encode("utf-8"))
        end = stop
    output.extend(source[end:])
    result = bytes(output)
    if digest(result) != recipe["result_hash"]:
        raise ValueError("repair result does not match reviewed outcome")
    return result


def preview(case, lab, analyzer: Path, run_behavior=False):
    expectation = case["expectations"]
    recipe = expectation.get("repair")
    if recipe is None:
        return {"id": case["id"], "status": "unsupported", "reason": "no reviewed repair recipe"}
    original = lab.read_source(lab.ROOT / case["original"])
    candidate = apply_recipe(original, recipe, lab.digest)
    fixed = lab.read_source(lab.ROOT / case["fixed"])
    if candidate != fixed or lab.digest(fixed) != expectation["fixed"]["source_hash"]:
        raise ValueError("candidate differs from reviewed behavioral fixture")
    with tempfile.TemporaryDirectory(prefix="state-lens-repair-") as temporary:
        work = Path(temporary)
        source = work / ("main" + lab.LANGUAGES[case["language"]][0])
        trees, analyses = {}, {}
        for variant, data in (("original", original), ("candidate", candidate)):
            source.write_bytes(data)
            tree = lab.parse_file(source, case["language"])
            if tree["parse_errors"]:
                raise ValueError(f"{variant} contains syntax errors")
            trees[variant] = tree["tree"]
            code, stdout, stderr = lab.execute([analyzer.resolve(), "scan", source])
            if code or stderr:
                raise ValueError(stderr or f"analysis failed: {code}")
            analyses[variant] = json.loads(stdout)
    patch = "".join(difflib.unified_diff(original.decode().splitlines(True), candidate.decode().splitlines(True), fromfile=case["original"], tofile=case["fixed"]))
    syntax_diff = "".join(difflib.unified_diff(trees["original"].splitlines(True), trees["candidate"].splitlines(True), fromfile="original AST", tofile="candidate AST"))
    behavior = lab.verify_fix(case) if run_behavior else {"status": "not-run"}
    return {"id": case["id"], "status": "preview", "patch": patch,
            "syntax_diff": syntax_diff, "analyses": analyses, "behavior": behavior,
            "limitations": ["AST text comparison does not establish binding-aware operation correspondence.",
                            "A reviewed fixture outcome is not a general correctness proof."]}
