import hashlib
import json
import re
import tempfile
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
            self.assertEqual(sorted(set(re.findall(r'@([a-z_]+)', path.read_text()))), rule['captures'])
            self.assertTrue(rule['related_cwes'])
        self.assertEqual(set(root.glob('*/mutated-state/*/*.scm')), registered)

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
