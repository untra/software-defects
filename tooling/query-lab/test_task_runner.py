import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import artifacts
import task_runner as runner

class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.binary=self.root/'analyzer';self.binary.write_bytes(b'fake')
        self.source=self.root/'a.ts';self.source.write_text('let a = 1;')
        self.packet={'schema_version':1,'stage':'extract','inputs':[{'path':str(self.source),'sha256':artifacts.digest(self.source.read_bytes())}],'tool':'scan','case':None,'source':str(self.source),'output_kind':'tool_result','behavioral_execution':False,'split':'development','groups':['family'],'configuration_hash':runner.configuration(self.binary),'limits':{'timeout_seconds':30,'output_bytes':10000}}
    def test_rejects_changed_source(self):
        self.source.write_text('changed')
        with self.assertRaisesRegex(ValueError,'stale input'):runner.check_packet(self.packet,self.binary)
    def test_rejects_unregistered_tool(self):
        self.packet['stage']='hypothesize'
        with self.assertRaisesRegex(ValueError,'tool not allowed'):runner.check_packet(self.packet,self.binary)
    def test_rejects_held_out_development(self):
        self.packet.update(stage='hypothesize',tool='submission',output_kind='hypothesis',split='held_out')
        with self.assertRaisesRegex(ValueError,'held-out'):runner.check_packet(self.packet,self.binary)
    def test_resume_does_not_rerun_unchanged_tool(self):
        with patch.object(runner.lab,'execute',return_value=(0,'{}','')) as execute:
            original=runner.run(self.packet,self.binary,self.root/'results')
            restored=runner.run(self.packet,self.binary,self.root/'results',resume=True)
            self.assertEqual(original,restored);self.assertEqual(execute.call_count,1)
    def test_failed_stage_blocks_chain(self):
        with patch.object(runner.lab,'execute',return_value=(1,'','failure')):
            result=runner.chain({'stages':[{'packet':self.packet,'depends_on':[]}]},self.binary,self.root/'results',None)
            self.assertEqual(result['status'],'failed')
