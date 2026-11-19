import json
from pathlib import Path
import tempfile
import unittest
import query_lab as lab

DEFECTS = ('async-io-misuse', 'race-conditions', 'file-network-access')

class SharedStateTests(unittest.TestCase):
    def test_all_languages_have_pairs_and_runtime_oracles(self):
        cases = [c for c in lab.corpus() if c['family'] == 'shared-state' and c['defect'] in DEFECTS]
        self.assertEqual({(c['defect'], c['language']) for c in cases}, {(d, l) for d in DEFECTS for l in lab.LANGUAGES})
        for case in cases:
            self.assertTrue((lab.ROOT / case['fixed']).is_file())
            self.assertEqual(case['expectations']['runtime']['status'], 'supported')

    @unittest.skipUnless((lab.CACHE / 'cli/bin/tree-sitter').is_file(), 'run setup')
    def test_structural_calls_ignore_application_names_and_strings(self):
        for lang in lab.LANGUAGES:
            wrap = {'python': lambda body: 'def work():\n    ' + body + '\n',
                    'go': lambda body: 'package main\nfunc work() { ' + body + ' }',
                    'rust': lambda body: 'fn work() { ' + body + '; }'}.get(lang, lambda body: 'function work() { ' + body + '; }')
            with tempfile.TemporaryDirectory() as temporary:
                path = Path(temporary) / ('sample' + lab.LANGUAGES[lang][0])
                for defect in DEFECTS:
                    query = lab.HERE / 'queries' / lang / 'shared-state' / defect / 'signals.scm'
                    name = {'async-io-misuse': 'task_call', 'race-conditions': 'synchronization_call', 'file-network-access': 'io_call'}[defect]
                    for application_name in ('persist', 'renamed'):
                        source = wrap(application_name + '()')
                        path.write_text(source)
                        result = lab.run_query(query, path)
                        start = source.index(application_name + '()')
                        self.assertEqual(lab.expanded(lab.capture_counts(result['matches'])), [{'name': name, 'range': [start, start + len(application_name) + 2]}])
                    path.write_text(wrap('return "persist()"'))
                    self.assertEqual(lab.run_query(query, path)['matches'], [])

    def test_tokio_runtime_is_locked_and_explicit(self):
        lock = (lab.TOKIO_RUNTIME / 'Cargo.lock').read_text()
        self.assertIn('name = "tokio"', lock)
        cases = [c for c in lab.corpus() if c.get('expectations', {}).get('runtime', {}).get('runner') == 'tokio']
        self.assertGreaterEqual(len(cases), 4)
        self.assertTrue(all(c['language'] == 'rust' for c in cases))

class SharedContractTests(unittest.TestCase):
    def test_live_analyzer_evidence_matches_schema(self):
        import jsonschema
        binary = lab.ROOT / 'tooling/state-lens/target/debug/state-lens'
        if not binary.is_file():
            self.skipTest('build State Lens first')
        schema = json.loads((lab.ROOT / 'tooling/state-lens/schemas/shared_state.schema.json').read_text())
        source = lab.ROOT / 'examples/shared-state/race-conditions/typescript/01_read_await_write.ts'
        report = json.loads(lab.checked([binary, 'scan', source]))
        findings = [f['shared_state'] for d in report['documents'] for f in d['findings'] if 'shared_state' in f]
        self.assertTrue(findings)
        for finding in findings:
            jsonschema.validate(finding, schema)
