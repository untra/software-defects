"""Corpus identities and independent expectations; never imported by the analyzer."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXTENSIONS = {".go": "go", ".rs": "rust", ".ts": "typescript", ".py": "python"}
COMPOUND = "compound-defects"


def identity(file):
    parts = Path(file).parts
    compound = len(parts) > 1 and parts[1] == COMPOUND
    if len(parts) != (6 if compound else 5):
        raise ValueError(f"Invalid corpus path: {file}")
    root, family = parts[:2]
    defect, language, name = parts[-3:]
    if root not in {"examples", "examples-fixed"} or EXTENSIONS.get(Path(name).suffix) != language:
        raise ValueError(f"Invalid corpus identity: {file}")
    return dict(root=root, family=family, defect=defect, language=language,
                compound_class=parts[2] if compound else None)


def manifest_path(file):
    return ROOT / Path(file).parent.parent / "manifest.json"


def expectations(file):
    info = identity(file)
    if not info["compound_class"]:
        return None, []
    manifest = json.loads(manifest_path(file).read_text())
    example = next(e for e in manifest["examples"] if e["file"] == "/".join(Path(file).parts[-2:]))
    rules = [f"{info['language']}.{i['family']}.{i['defect']}" for i in example["ingredients"]]
    rules.append(f"{info['language']}.{COMPOUND}.{info['compound_class']}.{info['defect']}")
    return manifest["category"], rules


def files():
    return sorted(p for root in ("examples", "examples-fixed") for p in (ROOT / root).rglob("*")
                  if p.suffix in EXTENSIONS and p.is_file())


def shard(file):
    info = identity(file)
    return "/".join(filter(None, (info["root"], info["family"], info["compound_class"])))


def shards():
    return sorted({shard(p.relative_to(ROOT)) for p in files()})


if __name__ == "__main__":
    print(json.dumps(shards()))


def validate_compounds():
    import sys
    sys.path.insert(0, str(ROOT / "src"))
    from content import DEFECTS
    families = {f["id"]: f for f in DEFECTS}
    base = {family: {d["slug"] for d in data["defects"]}
            for family, data in families.items() if family.endswith("-state")}
    classes = {d["slug"]: d for d in families[COMPOUND]["defects"]}
    pairings = {"nondeterminism": {"mutated-state", "shared-state"},
                "runtime-errors": {"mutated-state", "unstructured-state"},
                "data-corruption": {"shared-state", "unstructured-state"}}
    errors = []
    for path in sorted((ROOT / "examples" / COMPOUND).glob("*/*/manifest.json")):
        manifest = json.loads(path.read_text())
        label = str(path.relative_to(ROOT))
        cls, defect = path.parts[-3:-1]
        taxonomy = classes.get(cls, {})
        category = next((c["slug"] for c in taxonomy.get("categories", []) if defect in c["members"]), None)
        if not category or (manifest.get("family"), manifest.get("class"), manifest.get("defect"), manifest.get("category")) != (COMPOUND, cls, defect, category):
            errors.append(f"{label}: taxonomy mismatch")
        for example in manifest["examples"]:
            source_path = path.parent / example["file"]
            if not source_path.is_file() or source_path.resolve().parent.parent != path.parent.resolve():
                errors.append(f"{label}: invalid source {example['file']}")
                continue
            source = source_path.read_text()
            ingredients = example.get("ingredients", [])
            identities = {(i.get("family"), i.get("defect")) for i in ingredients}
            if len(ingredients) != 2 or len(identities) != 2 or {i[0] for i in identities} != pairings.get(cls):
                errors.append(f"{label}/{example['file']}: expected two distinct ingredients from {pairings.get(cls)}")
            for ingredient in ingredients:
                if ingredient.get("defect") not in base.get(ingredient.get("family"), set()):
                    errors.append(f"{label}: invalid base identity {ingredient}")
                if not ingredient.get("evidence") or any(not evidence or evidence not in source for evidence in ingredient["evidence"]):
                    errors.append(f"{label}/{example['file']}: missing ingredient evidence text")
            if not example.get("bug_lines") or any(type(line) is not int or not 1 <= line <= len(source.splitlines()) for line in example["bug_lines"]):
                errors.append(f"{label}/{example['file']}: invalid bug lines")
            if EXTENSIONS.get(source_path.suffix) != example["language"] or source_path.parent.name != example["language"]:
                errors.append(f"{label}/{example['file']}: invalid language")
    return errors
