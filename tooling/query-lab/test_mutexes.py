import json
from pathlib import Path
import tempfile
import unittest

import query_lab as lab

DEFECT = 'threading-mutexes'


class MutexTests(unittest.TestCase):
    def test_every_mutex_has_a_reviewed_pair(self):
        cases = [case for case in lab.corpus() if case['defect'] == DEFECT]
        self.assertEqual({c['language'] for c in cases}, set(lab.LANGUAGES))
        for case in cases:
            self.assertTrue((lab.ROOT / case['fixed']).is_file(), case['id'])
            runtime = case['expectations']['runtime']
            self.assertIn(runtime['status'], ('supported', 'unsupported'))
            if runtime['status'] == 'unsupported':
                self.assertIn('Browser Web Locks', runtime['reason'])
            for variant in ('original', 'fixed'):
                self.assertEqual(lab.digest((lab.ROOT / case[variant]).read_bytes()), case['expectations'][variant]['source_hash'])

    @unittest.skipUnless((lab.CACHE / 'cli/bin/tree-sitter').is_file(), 'run setup')
    def test_operation_boundaries_and_literal_controls(self):
        templates = {
            'go': ('package main\nfunc work() { gate.Lock() }\n', 'gate.Lock()', 'Lock'),
            'rust': ('fn work() { gate.lock(); }\n', 'gate.lock()', 'lock'),
            'python': ('def work():\n    gate.acquire()\n', 'gate.acquire()', 'acquire'),
            'javascript': ('function work() { gate.acquire(); }\n', 'gate.acquire()', 'acquire'),
            'typescript': ('function work() { gate.acquire(); }\n', 'gate.acquire()', 'acquire'),
        }
        for language, (source, operation, method) in templates.items():
            relative = Path(language) / 'shared-state' / DEFECT / 'boundaries.scm'
            query = lab.HERE / 'queries' / relative
            self.assertEqual(query.read_bytes(), (lab.ROOT / 'queries' / relative).read_bytes())
            with tempfile.TemporaryDirectory() as temporary:
                file = Path(temporary) / ('test' + lab.LANGUAGES[language][0])
                for receiver in ('gate', 'renamed'):
                    variant = source.replace('gate', receiver)
                    operation_variant = operation.replace('gate', receiver)
                    file.write_text(variant)
                    result = lab.run_query(query, file)
                    begin = variant.index(operation_variant)
                    method_start = begin + len(receiver) + 1
                    self.assertCountEqual(lab.expanded(lab.capture_counts(result['matches'])), [
                        {'name': 'mutex_call', 'range': [begin, begin + len(operation_variant)]},
                        {'name': 'mutex_method', 'range': [method_start, method_start + len(method)]},
                    ])
                file.write_text(source.replace(operation, '"' + operation + '"'))
                self.assertEqual(lab.run_query(query, file)['matches'], [])

    def test_metadata_exposes_the_structural_contract(self):
        rules = json.loads((lab.ROOT / 'queries/rules.json').read_text())['rules']
        for language in lab.LANGUAGES:
            rule = next(r for r in rules if r['language'] == language and r['defect'] == DEFECT)
            self.assertEqual(rule['captures'], ['mutex_method', 'mutex_call'])
            self.assertIn('Structural', rule['limitation'])

class PairOracleTests(unittest.TestCase):
    def test_explicit_original_oracle_is_checked_independently(self):
        from unittest.mock import patch
        case = {'id': 'test', 'original': 'before', 'fixed': 'after', 'expectations': {
            'original': {}, 'fixed': {}, 'runtime': {'status': 'supported', 'stdout_contains': ['correct']},
            'baseline': {'deterministic': True, 'exit_code': 0, 'stdout_contains': ['wrong']},
        }}
        def check(value):
            expected = 'wrong' if value['fixed'] == 'before' else 'correct'
            self.assertEqual(value['expectations']['runtime']['stdout_contains'], [expected])
            return {'status': 'passed', 'exit_code': 0}
        with patch.object(lab, 'verify_fix', side_effect=check):
            self.assertEqual(lab.verify_pair(case)['original']['status'], 'reproduced')
        self.assertEqual(case['expectations']['runtime']['stdout_contains'], ['correct'])

    def test_browser_pair_reports_unsupported_without_attempting_execution(self):
        from unittest.mock import patch
        with patch.object(lab, 'verify_fix', return_value={'status': 'unsupported', 'reason': 'browser'}) as check:
            result = lab.verify_pair({'id': 'browser', 'expectations': {}})
            self.assertEqual(result['status'], 'unsupported')
            check.assert_called_once()
