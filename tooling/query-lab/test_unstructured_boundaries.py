import json
from pathlib import Path
import tempfile
import unittest

import query_lab as lab

DEFECTS = ('lack-of-input-validation', 'unconstrained-inputs', 'duck-typing-without-contracts', 'missing-undefined-returns')


class UnstructuredBoundaryTests(unittest.TestCase):
    def test_production_matches_experimental_boundaries(self):
        rules = json.loads((lab.ROOT / 'queries/rules.json').read_text())['rules']
        for language in lab.LANGUAGES:
            for defect in DEFECTS:
                relative = Path(language) / 'unstructured-state' / defect
                self.assertEqual((lab.ROOT / 'queries' / relative / 'signals.scm').read_text(), (lab.HERE / 'queries' / relative / 'boundaries.scm').read_text())
                rule = next(rule for rule in rules if rule['language'] == language and rule['defect'] == defect)
                self.assertEqual(rule['evidence_level'], 'syntax-candidate')

    @unittest.skipUnless((lab.CACHE / 'cli/bin/tree-sitter').is_file(), 'run setup')
    def test_inputs_and_outputs_survive_renaming(self):
        templates = {
            'go': ('package main\nfunc NAME(items []int) int { return len(items) }', 'items []int', 'return len(items)'),
            'rust': ('fn NAME(items: &[i32]) -> usize { return items.len(); }', 'items: &[i32]', 'return items.len()'),
            'typescript': ('function NAME(items: number[]) { return items.length; }', 'items: number[]', 'return items.length;'),
            'javascript': ('function NAME(items) { return items.length; }', 'items', 'return items.length;'),
            'python': ('def NAME(items):\n    return len(items)\n', 'items', 'return len(items)'),
        }
        for language, (template, parameter, returned) in templates.items():
            for name in ('process', 'renamed'):
                source = template.replace('NAME', name)
                with tempfile.TemporaryDirectory() as temporary:
                    path = Path(temporary) / ('sample' + lab.LANGUAGES[language][0])
                    path.write_text(source)
                    for defect in ('lack-of-input-validation', 'unconstrained-inputs'):
                        result = lab.run_query(lab.HERE / 'queries' / language / 'unstructured-state' / defect / 'boundaries.scm', path)
                        begin = source.index(parameter)
                        self.assertEqual(lab.expanded(lab.capture_counts(result['matches'])), [{'name': 'unstructured_parameter', 'range': [begin, begin + len(parameter)]}])
                    result = lab.run_query(lab.HERE / 'queries' / language / 'unstructured-state/missing-undefined-returns/boundaries.scm', path)
                    captures = [capture for match in result['matches'] for capture in match['captures']]
                    self.assertEqual([c['text'] for c in captures if c['name'] == 'unstructured_return'], [returned])
                    self.assertEqual(len([c for c in captures if c['name'] == 'unstructured_function']), 1)

    @unittest.skipUnless((lab.CACHE / 'cli/bin/tree-sitter').is_file(), 'run setup')
    def test_application_names_are_not_resource_apis(self):
        snippets = {'go': 'package main\nfunc work() { count(); renamed() }', 'rust': 'fn work() { count(); renamed(); }', 'typescript': 'count(); renamed();', 'javascript': 'count(); renamed();', 'python': 'count()\nrenamed()\n'}
        for language, source in snippets.items():
            with tempfile.TemporaryDirectory() as temporary:
                path = Path(temporary) / ('sample' + lab.LANGUAGES[language][0])
                path.write_text(source)
                query = lab.HERE / 'queries' / language / 'unstructured-state/unconstrained-inputs/boundaries.scm'
                self.assertEqual(lab.run_query(query, path)['matches'], [])

    @unittest.skipUnless((lab.CACHE / 'cli/bin/tree-sitter').is_file(), 'run setup')
    def test_assertion_candidates_and_literal_negative_controls(self):
        snippets = {
            'go': ('package main\nfunc f(value any) { _ = value.(string) }', 'value.(string)'),
            'rust': ('fn f(value: &dyn Any) { value.downcast_ref::<String>(); }', 'value.downcast_ref::<String>()'),
            'typescript': ('const result = value as string;', 'value as string'),
            'javascript': ('value.save();', 'value.save()'),
            'python': ('result = cast(str, value)\n', 'cast(str, value)'),
        }
        negatives = {'go': 'package main\nvar text = "value.save()"', 'rust': 'const TEXT: &str = "value.save()";'}
        for language, (source, assertion) in snippets.items():
            with tempfile.TemporaryDirectory() as temporary:
                path = Path(temporary) / ('sample' + lab.LANGUAGES[language][0])
                path.write_text(source)
                query = lab.HERE / 'queries' / language / 'unstructured-state/duck-typing-without-contracts/boundaries.scm'
                report = lab.run_query(query, path)
                captures = [capture for match in report['matches'] for capture in match['captures'] if capture['name'] == 'unstructured_assertion']
                begin = source.index(assertion)
                self.assertEqual([(c['text'], c['range']) for c in captures], [(assertion, [begin, begin + len(assertion)])])
                path.write_text(negatives.get(language, '"value.save()"'))
                for defect in DEFECTS:
                    query = lab.HERE / 'queries' / language / 'unstructured-state' / defect / 'boundaries.scm'
                    self.assertEqual(lab.run_query(query, path)['matches'], [])
