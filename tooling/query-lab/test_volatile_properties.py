"""Collection mutation candidates, repair contracts and ownership boundaries."""
import contextlib
import io
from pathlib import Path
import tempfile
import unittest

import query_lab as lab

DEFECT = 'mutated-state/volatile-properties'


class VolatileTests(unittest.TestCase):
    def test_controls_and_known_false_positives(self):
        controls = {
            'typescript': ('items.push(1);', 'items.concat([1]);', 'const custom = {push() { return 1; }}; custom.push();'),
            'python': ('items.append(1)\n', 'sorted(items)\n', 'class Custom:\n    def append(self, value):\n        return value\nCustom().append(1)\n'),
            'rust': ('fn f() { items.push(1); }', 'fn f() { items.iter().collect(); }', 'fn f() { custom.push(1); }'),
            'go': ('package main\nfunc f() { copy(dst, src) }', 'package main\nfunc f() { len(src) }', 'package main\nfunc f() { copy := func(a, b int) {}; copy(1, 2) }'),
        }
        with tempfile.TemporaryDirectory() as temporary:
            for language, sources in controls.items():
                query = lab.HERE / 'queries' / language / DEFECT / 'signals.scm'
                path = Path(temporary) / ('source' + lab.LANGUAGES[language][0])
                for source, expected in zip(sources, (True, False, True)):
                    with self.subTest(language=language, source=source):
                        path.write_text(source)
                        result = lab.run_query(query, path)
                        captures = lab.expanded(lab.capture_counts(result['matches']))
                        self.assertEqual(bool(captures), expected)
                        if expected:
                            self.assertEqual({c['name'] for c in captures}, {'callee', 'receiver'})

    def test_expanded_syntax_boundaries(self):
        controls = {
            'typescript': [('items.join("");', False), ('new Map(items);', False)],
            'python': [('source.getvalue()\n', False), ('work.seek(0)\n', True)],
            'rust': [('fn f() { let entry = work.entry(1); }', True),
                     ('fn f() { let view = &items[1..]; }', False)],
            'go': [('package main\nimport s "slices"\nfunc f() { s.Sort(items) }', True),
                   ('package main\nfunc f() { view := items[1:] }', False)],
        }
        with tempfile.TemporaryDirectory() as temporary:
            for language, cases in controls.items():
                query = lab.HERE / 'queries' / language / DEFECT / 'signals.scm'
                path = Path(temporary) / ('source' + lab.LANGUAGES[language][0])
                for source, expected in cases:
                    with self.subTest(source=source):
                        path.write_text(source)
                        report = lab.run_query(query, path)
                        self.assertEqual(bool(report['matches']), expected)
                        if 'let entry' in source:
                            names = {c['name'] for m in report['matches'] for c in m['captures']}
                            self.assertEqual(names, {'access', 'receiver'})

    def test_conditional_and_cursor_semantics(self):
        from collections import deque
        from io import StringIO
        original = {'a': 1}
        work = original.copy()
        self.assertEqual(work.setdefault('a', 9), 1)
        self.assertEqual(original, {'a': 1})
        with self.assertRaises(KeyError):
            work.pop('missing')
        bounded = deque([1, 2], maxlen=2)
        copy = bounded.copy()
        copy.extend([3])
        self.assertEqual(list(copy), [2, 3])
        self.assertEqual(list(bounded), [1, 2])
        with self.assertRaises(IndexError):
            deque().popleft()
        source = StringIO('abc')
        source.seek(1)
        work = StringIO(source.getvalue())
        work.seek(source.tell())
        work.write('Z')
        self.assertEqual((work.getvalue(), work.tell()), ('aZc', 2))
        self.assertEqual((source.getvalue(), source.tell()), ('abc', 1))
        self.assertEqual(frozenset({1, 2}) | {3}, frozenset({1, 2, 3}))

    def test_rust_entry_and_utf8_boundaries(self):
        source = r'''use std::collections::HashMap;
fn main() {
    let mut source = HashMap::from([(1, 10)]);
    { let _entry = source.entry(2); }
    assert_eq!(source, HashMap::from([(1, 10)]));
    let mut work = source.clone();
    assert_eq!(*work.entry(1).or_insert(99), 10);
    work.entry(2).and_modify(|value| *value = 99);
    assert!(!work.contains_key(&2));
    let original = String::from("éab");
    let mut copy = original.clone();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| copy.remove(1)));
    assert!(failure.is_err());
    assert_eq!(original, "éab");
    let mut empty = Vec::<i32>::new();
    assert_eq!(empty.pop(), None);
    let drained: Vec<_> = empty.drain(..).collect();
    assert!(drained.is_empty());
}
'''
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'main.rs'
            path.write_text(source)
            executable = Path(directory) / 'program'
            lab.checked(['rustc', '--edition', '2021', path, '-o', executable], cwd=directory)
            lab.checked([executable], cwd=directory)

    def test_python_originals_fail_and_repairs_repeat(self):
        for path in (lab.ROOT / 'examples' / DEFECT / 'python').glob('*.py'):
            with self.subTest(path=path):
                with self.assertRaisesRegex(AssertionError, 'source changed'):
                    exec(path.read_text(), {})
                fixed = lab.ROOT / 'examples-fixed' / path.relative_to(lab.ROOT / 'examples')
                for _ in range(2):
                    with contextlib.redirect_stdout(io.StringIO()):
                        exec(fixed.read_text(), {})

    def test_python_empty_and_removal_semantics(self):
        source = []
        for operation in (lambda x: x.sort(), lambda x: x.reverse(), lambda x: x.clear()):
            work = source.copy()
            operation(work)
            self.assertEqual(source, [])
        for operation, error in ((lambda x: x.pop(), IndexError), (lambda x: x.remove(1), ValueError)):
            with self.assertRaises(error):
                operation(source.copy())
        source = [1, 1, 2]
        work = source.copy()
        work.remove(1)
        self.assertEqual(work, [1, 2])
        self.assertEqual(source, [1, 1, 2])

    def test_go_append_capacity_boundary(self):
        source = '''package main
import "slices"
func main() {
    original := []int{3, 1, 1, 2}
    shared := append(original[:2], 9)
    if original[2] != 9 || shared[2] != 9 { panic("expected shared storage") }
    independent := append(original[:2:2], 8)
    if original[2] != 9 || independent[2] != 8 { panic("expected allocation") }
    clone := slices.Clone(original)
    clone[0] = 7
    if original[0] != 3 { panic("clone aliased") }
}
'''
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'main.go'
            path.write_text(source)
            lab.checked(['go', 'run', path], cwd=directory)

    def test_missing_python_repairs(self):
        expected = {
            'this-mutation/python/01_class_attribute_through_self.py': '10 4\n',
            'this-mutation/python/02_init_calls_override.py': '5\n',
            'this-mutation/python/03_function_stored_on_instance.py': 'report for ada\n',
            'this-mutation/python/04_builder_forgets_self.py': 'pen\n',
            'increment-decrement-operators/python/01_float_mantissa.py': 'False\n',
            'increment-decrement-operators/python/02_bool_added_to_total.py': '1\n',
        }
        for relative, output in expected.items():
            with self.subTest(relative=relative), contextlib.redirect_stdout(io.StringIO()) as stream:
                exec((lab.ROOT / 'examples-fixed/mutated-state' / relative).read_text(), {})
                self.assertEqual(stream.getvalue(), output)
