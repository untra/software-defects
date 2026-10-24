import hashlib
import json
import unittest

import smoke


class SmokeBaselineTests(unittest.TestCase):
    def test_baseline_covers_every_corpus_file_at_its_current_hash(self):
        baseline = json.loads(smoke.BASELINE.read_text())["files"]
        files = {smoke.relative(path): path for path in smoke.corpus_files()}
        self.assertEqual(set(files), set(baseline))
        for name, path in files.items():
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), baseline[name]["source_hash"], name)

    def test_every_language_is_exercised(self):
        languages = {smoke.EXTENSIONS[path.suffix] for path in smoke.corpus_files()}
        self.assertEqual(languages, set(smoke.EXTENSIONS.values()))


class SmokeAuditTests(unittest.TestCase):
    def test_group_partition(self):
        files = smoke.corpus_files()
        grouped = smoke.groups(files)
        flattened = [file for group in grouped.values() for file in group]
        self.assertCountEqual(flattened, files)
        self.assertEqual(len(flattened), len(set(flattened)))
        self.assertEqual(len({file.parts[:2] for file in map(lambda p: p.relative_to(smoke.ROOT), files)}), 6)
        for folder, group in grouped.items():
            self.assertTrue(all(file.parent == folder for file in group))

    def test_row_schema_and_status(self):
        import smoke_report as audit
        row = audit.row('protocol', 'examples/mutated-state/var-let-mut/go/01_shadowed_err_in_if.go')
        self.assertEqual(set(row), audit.FIELDS)
        row['hinted_functions'] = 0
        audit.finish(row, [])
        self.assertEqual(row['status'], 'gap')
        self.assertFalse(audit.failed([row]))
        self.assertTrue(audit.failed([row], strict=True))
        row['problems'].append(audit.problem('log-warning', 'warning'))
        audit.finish(row)
        self.assertEqual(row['status'], 'fail')
        self.assertIn('server.rs', row['problems'][0]['where_to_look'])
        audit.validate(row)

    def test_manifest_half_open_lines(self):
        import smoke_report as audit
        row = audit.row('editor', 'examples/mutated-state/var-let-mut/go/01_shadowed_err_in_if.go')
        row['hinted_functions'] = 1
        audit.finish(row, [{'start': {'line': 11, 'character': 0}, 'end': {'line': 12, 'character': 0}}])
        self.assertEqual(row['bug_lines_missed'], [13])
        audit.finish(row, [{'start': {'line': 12, 'character': 0}, 'end': {'line': 12, 'character': 1}}])
        self.assertEqual(row['bug_lines_hit'], [13])
        self.assertEqual(row['status'], 'pass')

    def test_fixed_files_are_not_gaps(self):
        import smoke_report as audit
        row = audit.row('editor', 'examples-fixed/mutated-state/var-let-mut/go/01_shadowed_err_in_if.go')
        row['hinted_functions'] = 0
        audit.finish(row, [])
        self.assertEqual(row['status'], 'pass')
        self.assertEqual(row['bug_lines_missed'], [])

    def test_merge_missing_and_duplicate_rows(self):
        import tempfile
        from pathlib import Path
        import smoke_report as audit
        with tempfile.TemporaryDirectory() as directory:
            row = audit.finish(audit.row('protocol', smoke.relative(smoke.corpus_files()[0])))
            audit.write(directory, [row])
            report = Path(directory) / 'smoke-report.json'
            with self.assertRaisesRegex(ValueError, 'Duplicate'):
                audit.merge([report, report], ['protocol'])
            rows = audit.merge([report], ['protocol', 'editor'])
            self.assertEqual(len(rows), 2 * len(smoke.corpus_files()))
            self.assertTrue(audit.failed(rows))

    def test_negative_positions(self):
        self.assertTrue(smoke.range_problems('trigger', {'start': {'line': -1, 'character': 0}, 'end': {'line': 0, 'character': -1}}, ['abc']))

    def test_response_routing_and_log_warning(self):
        import queue
        import time
        client = smoke.Client.__new__(smoke.Client)
        client.pending = []
        client.inbox = queue.Queue()
        client.errors = []
        client.latest_diagnostics = {}
        client.deadline = time.monotonic() + 1
        client.inbox.put({'method': 'window/showMessage', 'params': {'type': 2, 'message': 'warning'}})
        client.inbox.put({'id': 2, 'result': 'second'})
        client.inbox.put({'id': 1, 'result': 'first'})
        self.assertEqual(client.response(1)['result'], 'first')
        self.assertEqual(client.response(2)['result'], 'second')
        self.assertEqual(client.errors[0]['category'], 'log-warning')
        client.deadline = time.monotonic() - 1
        with self.assertRaises(TimeoutError):
            client.response(3)

    def test_content_modified_requires_later_mutation(self):
        from unittest.mock import Mock
        import smoke_report as audit
        for later, allowed, failures in [(True, True, 0), (False, True, 1), (True, False, 1)]:
            client = Mock()
            client.requests = {1: ('hover', 1)}
            client.mutation = 2 if later else 1
            client.response.return_value = {'error': {'code': smoke.CONTENT_MODIFIED, 'message': 'stale'}}
            row = audit.row('protocol', smoke.relative(smoke.corpus_files()[0]))
            self.assertIsNone(smoke.result(client, 1, row, allowed))
            self.assertEqual(len(row['problems']), failures)


    def test_failure_never_updates_baseline(self):
        import concurrent.futures
        import contextlib
        import io
        from unittest.mock import Mock, patch
        import smoke_report as audit
        path = smoke.corpus_files()[0]
        row = audit.row('protocol', smoke.relative(path))
        row['problems'].append(audit.problem('error', 'injected'))
        audit.finish(row)
        future = concurrent.futures.Future()
        future.set_result(([row], {smoke.relative(path): {}}))
        pool = Mock()
        pool.submit.return_value = future
        manager = Mock()
        manager.__enter__ = Mock(return_value=pool)
        manager.__exit__ = Mock(return_value=False)
        with patch.object(smoke, 'corpus_files', return_value=[path]), \
             patch.object(smoke.concurrent.futures, 'ProcessPoolExecutor', return_value=manager), \
             patch.object(audit, 'atomic_json') as write, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(smoke.run(smoke.DEFAULT_BINARY, True, jobs=1), 1)
            write.assert_not_called()

    def test_group_timeout_restarts_server_and_keeps_rows(self):
        from unittest.mock import Mock, patch
        paths = [group[0] for group in list(smoke.groups(smoke.corpus_files()).values())[:2]]
        client = Mock()
        client.errors = []
        client.shutdown.return_value = 0
        with patch.object(smoke, 'initialize', return_value=client) as initialize, \
             patch.object(smoke, 'check_group', side_effect=[TimeoutError('injected'), None]):
            rows, _ = smoke.run_shard(smoke.DEFAULT_BINARY, 'examples/mutated-state', paths, {}, False, None)
        self.assertEqual(initialize.call_count, 2)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0]['problems'][0]['category'], 'timeout')
        self.assertEqual(client.terminate.call_count, 2)


if __name__ == "__main__":
    unittest.main()
