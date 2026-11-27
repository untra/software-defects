import hashlib
import json
import re
import tempfile
import subprocess
import unittest
from pathlib import Path

import corpus


class CorpusTests(unittest.TestCase):
    def test_inventory_covers_every_source_without_changing_ground_truth(self):
        inventory = json.loads((corpus.ROOT / 'tooling/state-lens/coverage.json').read_text())
        authored = {str(source.relative_to(corpus.ROOT)): source for _, _, source in corpus.cases()}
        self.assertEqual(set(authored), {case['file'] for case in inventory['cases']})
        for case in inventory['cases']:
            self.assertEqual(hashlib.sha256(authored[case['file']].read_bytes()).hexdigest(), case['source_hash'])
            self.assertEqual(case['bug_detection'], 'unsupported')

    def test_bug_line_overlap_uses_utf8_and_same_document_evidence(self):
        source = "é = 1\nboom()\n"
        doc = {"uri": "file:///example.py"}
        finding = {"trigger": {"start": 0, "end": 6}, "evidence": [
            {"uri": "file:///other.py", "range": {"start": 7, "end": 13}}]}
        self.assertFalse(corpus.overlaps_bug_lines(finding, doc, source, [2]))
        finding["evidence"][0]["uri"] = doc["uri"]
        self.assertTrue(corpus.overlaps_bug_lines(finding, doc, source, [2]))
        finding["evidence"] = []
        finding["trigger"] = {"start": 7, "end": 13}
        self.assertTrue(corpus.overlaps_bug_lines(finding, doc, source, [2]))

    def test_family_filter_includes_nested_compounds(self):
        cases = list(corpus.cases("compound-defects"))
        self.assertTrue(cases)
        self.assertTrue(all(m["family"] == "compound-defects" for m, _, _ in cases))

    def test_isolated_batch_does_not_share_bindings_between_documents(self):
        with tempfile.TemporaryDirectory(prefix='state-lens-isolated-') as directory:
            first, second = Path(directory) / 'first.js', Path(directory) / 'second.js'
            first.write_text('const items = []; console.log(items[0].id);')
            second.write_text('const items = [1]; console.log(items[0]);')
            output = subprocess.check_output([str(corpus.DEFAULT_BINARY), 'scan-isolated'], input=json.dumps([str(first), str(second)]).encode())
            reports = [json.loads(line) for line in output.splitlines()]
            self.assertEqual(len(reports), 2)
            self.assertTrue(all(len(report['documents']) == 1 for report in reports))
            self.assertTrue(any(f['defect'] == 'index-out-of-bounds' for f in reports[0]['documents'][0]['findings']))
            self.assertFalse(any(f['defect'] == 'index-out-of-bounds' for f in reports[1]['documents'][0]['findings']))

    def test_p0_acceptance_targets(self):
        inventory = json.loads((corpus.ROOT / 'tooling/state-lens/coverage.json').read_text())
        defects = {'threading-mutexes', 'race-conditions', 'resource-contention', 'cache-invalidation', 'insecure-deserialization', 'index-out-of-bounds', 'type-errors'}
        for row in inventory['summary']:
            if row['language'] != 'all':
                continue
            if row['defect'] in defects:
                self.assertGreaterEqual(row['recall'], 0.70, row['defect'])
                self.assertEqual(row['fixed_missing'], 0, row['defect'])
                self.assertLess(row['fixed_risk_rate'], 0.25, row['defect'])
            if row['defect'] == 'divide-by-zero':
                self.assertEqual(row['fixed_risk_flagged'], 0)

    def test_query_metadata_matches_embedded_capture_contract(self):
        root = corpus.ROOT / 'queries'
        metadata = json.loads((root / 'rules.json').read_text())
        registered = set()
        identifiers = set()
        for rule in metadata['rules']:
            self.assertNotIn(rule['id'], identifiers)
            identifiers.add(rule['id'])
            path = root / rule['query']
            registered.add(path)
            self.assertEqual(sorted(set(re.findall(r'@([a-z][a-z_]*)', path.read_text()))), sorted(rule['captures']))
            self.assertTrue(rule['related_cwes'])
        self.assertEqual(set(root.glob('*/*/*/*.scm')), registered)

    def test_blind_export_keeps_all_labels_outside_sources(self):
        with tempfile.TemporaryDirectory(prefix='state-lens-blind-test-') as directory:
            root = Path(directory)
            output, private = root / 'source', root / 'private.json'
            corpus.blind(output, private)
            mapping = json.loads(private.read_text())['cases']
            self.assertEqual(len(mapping), len(list(corpus.cases())))
            self.assertFalse(list(output.rglob('*.json')))
            self.assertFalse(list(output.rglob('*.md')))
            for case in mapping:
                source = output / case['file']
                self.assertEqual(source.stem, 'main')
                self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), case['source_hash'])
            with self.assertRaises(ValueError):
                corpus.blind(output, private)

    def test_private_mapping_cannot_be_exported_to_detector(self):
        with tempfile.TemporaryDirectory(prefix='state-lens-mapping-test-') as directory:
            root = Path(directory)
            with self.assertRaises(ValueError):
                corpus.blind(root / 'source', root / 'source/private.json')


if __name__ == '__main__':
    unittest.main()
