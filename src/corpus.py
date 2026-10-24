"""Loads the runnable example corpus (../examples) for rendering on defect pages.

Each defect folder carries a manifest.json with ground truth for its files;
repaired counterparts live in ../examples-fixed at the same relative path, and
Query Lab sidecars in ../tooling/query-lab/cases hold the repair rationale.
A manifest that names a missing file or an out-of-range bug line fails the build.
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(HERE, ".."))
EXAMPLES_DIR = os.path.join(REPO_ROOT, "examples")
FIXED_DIR = os.path.join(REPO_ROOT, "examples-fixed")
CASES_DIR = os.path.join(REPO_ROOT, "tooling", "query-lab", "cases")
MANIFEST = "manifest.json"

CORPUS_LANGUAGES = {"go": "Go", "rust": "Rust", "typescript": "TypeScript", "python": "Python"}


class CorpusError(ValueError):
    pass


def _read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


def _optional(path):
    return _read(path) if os.path.isfile(path) else None


def _rationale(family, slug, rel):
    stem, _ = os.path.splitext(rel)
    sidecar = os.path.join(CASES_DIR, family, slug, stem + ".json")
    if not os.path.isfile(sidecar):
        return None
    return json.loads(_read(sidecar)).get("rationale")


def _example(family, slug, entry):
    rel = entry["file"]
    path = os.path.join(EXAMPLES_DIR, family, slug, rel)
    if not os.path.isfile(path):
        raise CorpusError(f"{family}/{slug}/{MANIFEST} names missing file {rel}")
    source = _read(path)
    line_count = len(source.splitlines())
    bad = [n for n in entry["bug_lines"] if not 1 <= n <= line_count]
    if bad:
        raise CorpusError(f"{family}/{slug}/{rel}: bug_lines {bad} outside 1..{line_count}")
    return {
        **entry,
        "path": f"examples/{family}/{slug}/{rel}",
        "source": source,
        "fixed_source": _optional(os.path.join(FIXED_DIR, family, slug, rel)),
        "fix_rationale": _rationale(family, slug, rel),
    }


def _language_order(langs):
    known = [lg for lg in CORPUS_LANGUAGES if lg in langs]
    return known + sorted(lg for lg in langs if lg not in CORPUS_LANGUAGES)


def language_label(lang):
    return CORPUS_LANGUAGES.get(lang, lang.title())


def load_corpus():
    """{(family, slug): {"cwes": [...], "languages": [(lang, [example, ...]), ...]}}"""
    corpus = {}
    if not os.path.isdir(EXAMPLES_DIR):
        return corpus
    for family in sorted(os.listdir(EXAMPLES_DIR)):
        family_dir = os.path.join(EXAMPLES_DIR, family)
        if not os.path.isdir(family_dir):
            continue
        for slug in sorted(os.listdir(family_dir)):
            manifest_path = os.path.join(family_dir, slug, MANIFEST)
            if not os.path.isfile(manifest_path):
                continue
            manifest = json.loads(_read(manifest_path))
            by_language = {}
            for entry in manifest["examples"]:
                by_language.setdefault(entry["language"], []).append(_example(family, slug, entry))
            corpus[(family, slug)] = {
                "cwes": manifest.get("cwes", []),
                "languages": [(lg, sorted(by_language[lg], key=lambda x: x["file"]))
                              for lg in _language_order(by_language)],
            }
    return corpus
