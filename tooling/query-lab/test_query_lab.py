import argparse
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import query_lab as lab


class AdapterTests(unittest.TestCase):
    def test_unicode_multiline_crlf_and_repeated_captures(self):
        source = 'α`x\r\nsecond'.encode()
        output = '/case.ts\n  pattern: 2\n    capture: 0 - value, start: (0, 0), end: (0, 4), text: `α`x`\n    capture: body, start: (0, 0), end: (1, 6)\n  pattern: 2\n    capture: 0 - value, start: (0, 0), end: (0, 4), text: `ignored`\n'
        matches = lab.decode_matches(output, source, Path('/case.ts'))
        self.assertEqual(matches[0]['captures'][0]['text'], 'α`x')
        self.assertEqual(matches[0]['captures'][1]['text'], source.decode())
        self.assertEqual(lab.capture_counts(matches)[('value', 0, 4)], 2)

    def test_empty_results_and_unexpected_output(self):
        self.assertEqual(lab.decode_matches('/case.ts\n', b'', Path('/case.ts')), [])
        for suffix in ('WARNING: Query exceeded maximum number of in-progress captures!',
                       '    capture: x, start: (0, 0), end: (0, 1)', 'garbage'):
            with self.assertRaises(lab.LabError):
                lab.decode_matches('/case.ts\n' + suffix, b'x', Path('/case.ts'))

    def test_invalid_ranges_and_utf8_boundaries(self):
        for start, end in ((2, 1), (0, 99), (1, 2)):
            with self.assertRaises((lab.LabError, UnicodeDecodeError)):
                lab.decode_matches(f'/case.ts\n  pattern: 0\n    capture: x, start: (0, {start}), end: (0, {end})\n', 'α'.encode(), Path('/case.ts'))

    def test_predicates_cannot_silently_be_ignored(self):
        lab.validate_predicates('((identifier) @x (#eq? @x "#custom?")) ; (#custom? @x)')
        for predicate in ('#custom?', '#set!', '.custom?'):
            with self.assertRaises(lab.LabError):
                lab.validate_predicates(f'((identifier) @x ({predicate} @x))')

    def test_subprocess_timeout_and_output_limit(self):
        with self.assertRaisesRegex(lab.LabError, 'timed out'):
            lab.execute([sys.executable, '-c', 'import time; time.sleep(10)'], timeout=0.1)
        with patch.object(lab, 'MAX_OUTPUT_BYTES', 100):
            with self.assertRaisesRegex(lab.LabError, 'output limit'):
                lab.execute([sys.executable, '-c', 'print("x" * 1000)'])

    def test_no_match_selection_is_not_success(self):
        with self.assertRaises(lab.LabError):
            lab.select(argparse.Namespace(case='does/not/exist'))

    def test_source_limit_and_wrong_cli_version(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)
            source = path / 'large.ts'
            with source.open('wb') as file:
                file.truncate(lab.MAX_SOURCE_BYTES + 1)
            with self.assertRaisesRegex(lab.LabError, 'source exceeds'):
                lab.read_source(source)
            binary = path / 'cli/bin/tree-sitter'
            binary.parent.mkdir(parents=True)
            binary.touch()
            with patch.object(lab, 'CACHE', path), patch.object(lab, 'checked', return_value='tree-sitter 0.26.0'):
                with self.assertRaisesRegex(lab.LabError, 'wrong CLI version'):
                    lab.cli()


class CorpusTests(unittest.TestCase):
    def test_every_supported_manifest_entry_has_an_independent_fixed_pair(self):
        cases = lab.corpus()
        self.assertEqual(len(cases), 284)
        self.assertEqual(len({c['id'] for c in cases}), len(cases))
        for case in cases:
            with self.subTest(case=case['id']):
                side = case['expectations']
                self.assertEqual(side['schema_version'], lab.SCHEMA_VERSION)
                self.assertTrue(side['rationale'])
                self.assertTrue(side['analysis_limit'])
                for variant in ('original', 'fixed'):
                    data = lab.read_source(lab.ROOT / case[variant])
                    self.assertEqual(lab.digest(data), side[variant]['source_hash'])
                    self.assertTrue(side[variant]['queries'])
                    for spec in side[variant]['queries']:
                        self.assertTrue((lab.ROOT / spec['query']).is_file())
                        for capture in spec['captures']:
                            start, end = capture['range']
                            self.assertLessEqual(0, start)
                            self.assertLess(start, end)
                            self.assertLessEqual(end, len(data))
                            data[start:end].decode('utf-8')
                self.assertNotEqual(side['original']['source_hash'], side['fixed']['source_hash'])
                if side['runtime']['status'] == 'supported':
                    self.assertTrue(side['runtime'].get('stdout') or side['runtime'].get('stdout_contains'))

    def test_new_languages_are_catalogued_without_execution_support(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            directory = root / 'examples/family/defect'
            directory.mkdir(parents=True)
            (directory / 'manifest.json').write_text(json.dumps({
                'family': 'family', 'defect': 'defect',
                'examples': [{'file': 'python/example.py', 'language': 'python', 'bug_lines': [1]}],
            }))
            with patch.object(lab, 'ROOT', root), patch.object(lab, 'SIDECARS', root / 'cases'):
                self.assertEqual(lab.corpus(), [])
                unsupported = lab.corpus(include_unsupported=True)
                self.assertEqual(unsupported[0]['status'], 'unsupported-language')

    def test_stale_expectations_fail_before_running_queries(self):
        case = lab.corpus()[0]
        case['expectations']['original']['source_hash'] = 'stale'
        with self.assertRaisesRegex(lab.LabError, 'stale expectations'):
            lab.test_case(case)

    def test_matrix_is_complete_and_experimental(self):
        rules = json.loads((lab.HERE / 'queries/rules.json').read_text())['rules']
        cells = {(c['language'], c['family'], c['defect']) for c in lab.corpus()}
        self.assertEqual(len(rules), 84)
        self.assertEqual(cells, {(r['language'], r['family'], r['defect']) for r in rules})
        for rule in rules:
            self.assertTrue(rule['query'].startswith('tooling/query-lab/queries/'))
            self.assertEqual(rule['grammar_version'], lab.LANGUAGES[rule['language']][1])


@unittest.skipUnless((lab.CACHE / 'cli/bin/tree-sitter').is_file(), 'run query_lab.py setup for integration tests')
class NativeTests(unittest.TestCase):
    def test_live_query_edit_and_compile_diagnostics(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)
            source, query = path / 'source.ts', path / 'query.scm'
            source.write_text('const text = "α`";\r\nconst other = 1;\r\n')
            query.write_text('(string) @text')
            first = lab.run_query(query, source)
            self.assertEqual(first['matches'][0]['captures'][0]['text'], '"α`"')
            query.write_text('(number) @number')
            second = lab.run_query(query, source)
            self.assertNotEqual(first['query_hash'], second['query_hash'])
            self.assertEqual(second['matches'][0]['captures'][0]['text'], '1')
            for invalid in ('(nonexistent_node) @x', '(identifier wrong_field: (identifier)) @x', '('):
                query.write_text(invalid)
                with self.assertRaises(lab.LabError):
                    lab.run_query(query, source)
            source.write_text('function {')
            self.assertTrue(lab.parse_file(source)['parse_errors'])

    def test_every_pack_has_a_positive_and_benign_negative_control(self):
        rules = json.loads((lab.HERE / 'queries/rules.json').read_text())['rules']
        with tempfile.TemporaryDirectory() as temporary:
            for rule in rules:
                with self.subTest(rule=rule['id']):
                    lang = rule['language']
                    query = lab.ROOT / rule['query']
                    if rule['defect'] == 'divide-by-zero':
                        expression, capture, name = 'value / divisor', '/', 'operator'
                    elif lang == 'rust' and rule['defect'] == 'var-let-mut':
                        expression, capture, name = 'let value = 1', 'let', 'binding'
                    else:
                        import re
                        api = re.search(r'#any-of\? @callee "([^"]+)"', query.read_text())[1]
                        expression, capture, name = f'object.{api}()', api, 'callee'
                    body = expression + ';'
                    if lang == 'go':
                        positive = 'package main\nfunc sample() { ' + body + ' }\n'
                        negative = 'package main\nfunc sample() { _ = "' + expression + '" }\n'
                    elif lang == 'rust':
                        positive = 'fn sample() { ' + body + ' }\n'
                        negative = 'fn sample() { const VALUE: &str = "' + expression + '"; }\n'
                    else:
                        positive = 'function sample() { ' + body + ' }\n'
                        negative = 'function sample() { return "' + expression + '"; }\n'
                    source = Path(temporary) / ('control' + lab.LANGUAGES[lang][0])
                    source.write_text(positive)
                    report = lab.run_query(query, source)
                    position = positive.index(expression) + expression.index(capture)
                    start = len(positive[:position].encode())
                    self.assertEqual(lab.expanded(lab.capture_counts(report['matches'])), [{'name': name, 'range': [start, start + len(capture)]}])
                    source.write_text(negative)
                    self.assertEqual(lab.run_query(query, source)['matches'], [])


if __name__ == '__main__':
    unittest.main()
