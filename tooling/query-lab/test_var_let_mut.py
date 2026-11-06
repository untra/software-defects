"""Behavioral edge cases for the reviewed var-let-mut repairs."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest

import query_lab as lab

DEFECT = Path('mutated-state/var-let-mut')
FIXED = lab.ROOT / 'examples-fixed' / DEFECT


class RepairTests(unittest.TestCase):
    def python_fixture(self, filename):
        namespace = {}
        with contextlib.redirect_stdout(io.StringIO()):
            exec((FIXED / 'python' / filename).read_text(), namespace)
        return namespace

    def test_python_search_and_repeated_calls(self):
        search = self.python_fixture('02_loop_name_leaks.py')['selected']
        self.assertEqual(search([], 'cam'), 'cam is missing')
        self.assertEqual(search(['ada', 'cam', 'ben'], 'cam'), 'cam')
        tax = self.python_fixture('03_assigned_name_is_local.py')['apply_tax']
        self.assertEqual([tax(0), tax(0.1), tax(0.1)], [40, 44, 44])
        rate = self.python_fixture('04_module_global_rebound.py')['tax']
        self.assertEqual([rate(100), rate(100, 0.2), rate(100)], [5, 20, 5])
        transform = self.python_fixture('05_accumulator_leaks_across_calls.py')['filter_and_transform']
        self.assertEqual(transform([]), [])
        self.assertEqual(transform([50]), [])
        self.assertEqual(transform([30, 25]), [50])
        self.assertEqual(transform([30, 25]), [50])

    def run_fixture(self, relative, assertions):
        language = relative.split('/')[0]
        source = (FIXED / relative).read_text()
        with tempfile.TemporaryDirectory(prefix='mutation-edges-') as directory:
            work = Path(directory)
            path = work / ('main' + Path(relative).suffix)
            if language == 'go':
                source = source.replace('func main()', 'func exampleMain()')
                source += '\nfunc main() {\n' + assertions + '\n}\n'
                path.write_text(source)
                lab.checked(['go', 'build', '-o', work / 'program', path], cwd=work)
                command = [work / 'program']
            elif language == 'rust':
                source = source.replace('fn main()', 'fn example_main()')
                path.write_text(source + '\nfn main() {\n' + assertions + '\n}\n')
                lab.checked(['rustc', '--edition', '2021', path, '-o', work / 'program'], cwd=work)
                command = [work / 'program']
            else:
                path.write_text(source.replace('main();', '') + assertions)
                command = ['node', path]
            lab.checked(command, cwd=work)

    def test_rust_fixed_threshold(self):
        self.run_fixture('rust/03_threshold_drifts.rs', '''
assert_eq!(process_signals(&[]), vec![]);
assert_eq!(process_signals(&[10, 9]), vec![]);
assert_eq!(process_signals(&[20, 15, 30]), vec![40, 30, 60]);
''')

    def test_go_identity_and_recovery(self):
        self.run_fixture('go/05_reused_task_pointer.go', '''
if len(buildTaskRegistry(nil)) != 0 { panic("empty registry") }
tasks := buildTaskRegistry([]string{"a", "b"})
tasks[0].Name = "changed"
if tasks[1].Name != "b" || tasks[0] == tasks[1] { panic("aliased tasks") }
''')
        self.run_fixture('go/03_defer_overwrites_named_return.go', '''
remaining, err := withdraw(100, 30)
if remaining != 70 || err != nil { panic("successful withdrawal") }
remaining, err = withdraw(100, 101)
if remaining != 100 || err == nil { panic("recovery") }
''')

    def test_async_context(self):
        self.run_fixture('typescript/05_async_id_capture.ts', '''
async function check() {
  const logs: string[] = [];
  console.log = (message: string) => { logs.push(message); };
  if ((await fetchItems([])).length !== 0) throw new Error("empty input");
  const data = await fetchItems([2, 1]);
  if (data.join(",") !== "2,1" || logs.join("|") !== "Processed for ID: 2|Processed for ID: 1") {
    throw new Error("request context mismatch");
  }
}
check();
''')

    def test_pair_metadata(self):
        manifest = json.loads((lab.ROOT / 'examples' / DEFECT / 'manifest.json').read_text())
        self.assertEqual(len(manifest['examples']), 18)
        for entry in manifest['examples']:
            relative = DEFECT / entry['file']
            side = json.loads((lab.SIDECARS / relative.with_suffix('.json')).read_text())
            for variant, root in [('original', 'examples'), ('fixed', 'examples-fixed')]:
                source = (lab.ROOT / root / relative).read_bytes()
                self.assertEqual(hashlib.sha256(source).hexdigest(), side[variant]['source_hash'], str(relative))
                if variant == 'original':
                    self.assertTrue(all(0 < line <= len(source.splitlines()) for line in entry['bug_lines']))


if __name__ == '__main__':
    unittest.main()
