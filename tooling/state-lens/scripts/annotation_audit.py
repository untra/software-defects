"""Inventory detector evidence and fixed counterparts without claiming bug accuracy."""
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
LANGUAGES = ('go', 'rust', 'typescript', 'javascript', 'python')


def audit():
    coverage = json.loads((ROOT / 'tooling/state-lens/coverage.json').read_text())['cases']
    rules = json.loads((ROOT / 'queries/rules.json').read_text())['rules']
    bases = []
    compounds = []
    for path in sorted((ROOT / 'examples').rglob('manifest.json')):
        manifest = json.loads(path.read_text())
        if manifest.get('family') == 'compound-defects':
            for example in manifest.get('examples', []):
                ingredients = example.get('ingredients', [])
                language = example['language']
                availability = []
                for ingredient in ingredients:
                    matches = [c for c in coverage if c.get('language') == language and c.get('defect') == ingredient['defect'] and c.get('family') == ingredient['family']]
                    availability.append({**ingredient, 'coverage_statuses': dict(Counter(c.get('susceptibility_status', 'unknown') for c in matches)), 'bug_detection_statuses': dict(Counter(c.get('bug_detection', 'unknown') for c in matches))})
                compounds.append({'manifest': str(path.relative_to(ROOT)), 'file': example['file'], 'ingredients': availability, 'readiness': 'requires recipe-specific relationship and range validation'})
            continue
        for language in LANGUAGES:
            examples = [e for e in manifest.get('examples', []) if e['language'] == language]
            cases = [c for c in coverage if c.get('language') == language and c.get('defect') == manifest['defect'] and c.get('family') == manifest['family']]
            missing = [e['file'] for e in examples if not (ROOT / 'examples-fixed' / path.parent.relative_to(ROOT / 'examples') / e['file']).exists()]
            bases.append({'family': manifest['family'], 'defect': manifest['defect'], 'language': language, 'examples': len(examples), 'fixed_counterparts': len(examples)-len(missing), 'missing_fixes': missing, 'queries': [r['query'] for r in rules if r['language'] == language and r['defect'] == manifest['defect']], 'coverage_statuses': dict(Counter(c.get('susceptibility_status', 'unknown') for c in cases)), 'bug_detection_statuses': dict(Counter(c.get('bug_detection', 'unknown') for c in cases)), 'range_support': 'linked mutated denominator only' if manifest['defect'] == 'divide-by-zero' else 'source only unless linked by mutable-denominator resolver'})
    return {'schema_version': 1, 'limitations': ['Coverage ledger is recorded evidence, not freshly measured accuracy.', 'Fixed counterpart presence does not establish correctness.', 'Catalog recipes are not enabled by ingredient availability alone.'], 'implemented_recipes': ['mutable-denominator: exact same-function mutation evidence linked by denominator analysis; highlights the division expression'], 'bases': bases, 'compounds': compounds}


if __name__ == '__main__':
    print(json.dumps(audit(), indent=2))
