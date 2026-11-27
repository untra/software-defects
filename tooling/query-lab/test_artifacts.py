import unittest
import artifacts

HASH='0'*64

def graph():
    return artifacts.envelope('state_graph',{'sources':[{'uri':'file:///a','source_hash':HASH,'byte_length':3}],'nodes':[{'id':'n','kind':'read','location':{'uri':'file:///a','source_hash':HASH,'start':0,'end':1}}],'edges':[],'limitations':[]},[],HASH,version=1)

class ArtifactTests(unittest.TestCase):
    def test_graph_roundtrip(self): artifacts.validate(graph())
    def test_hash_tampering(self):
        value=graph(); value['payload']['limitations'].append('changed')
        with self.assertRaisesRegex(ValueError,'hash mismatch'): artifacts.validate(value)
    def test_dangling_edge(self):
        value=graph(); value['payload']['edges']=[{'from':'n','to':'missing','kind':'dependency','evidence':[]}];value['artifact_id']=artifacts.identity(value)
        with self.assertRaisesRegex(ValueError,'dangling'): artifacts.validate(value)
    def test_out_of_bounds(self):
        value=graph();value['payload']['nodes'][0]['location']['end']=10;value['artifact_id']=artifacts.identity(value)
        with self.assertRaisesRegex(ValueError,'span'): artifacts.validate(value)
    def test_unknown_kind(self):
        with self.assertRaisesRegex(ValueError,'unknown'): artifacts.validate({'kind':'../../secret'})

class CanonicalGraphTests(unittest.TestCase):
    def payload(self):
        import json
        return json.loads((artifacts.SCHEMAS.parent/'tests/fixtures/state-contract.json').read_text())

    def test_shared_fixture_and_canonical_identity(self):
        import json
        value=artifacts.envelope('state_graph',self.payload(),[],HASH)
        pretty=json.dumps(value,indent=4).encode()
        compact=artifacts.canonical(value)
        self.assertNotEqual(artifacts.digest(pretty),artifacts.digest(compact))
        self.assertEqual(artifacts.identity(json.loads(pretty)),artifacts.identity(json.loads(compact)))
        artifacts.validate(json.loads(compact))

    def test_all_collections_reject_duplicate_identities(self):
        for kind in ('operations','bindings','allocations','blocks','conditions','access_paths','call_contexts','evidence','obligations'):
            with self.subTest(kind=kind):
                payload=self.payload(); graph=payload['graphs'][0]
                graph[kind].append(graph[kind][0])
                with self.assertRaisesRegex(ValueError,'duplicate'): artifacts.envelope('state_graph',payload,[],HASH)

    def test_nested_references_and_evidence(self):
        mutations = [
            lambda g: g['conditions'][0]['evidence'].append('missing'),
            lambda g: g['access_paths'][0].update(root='missing'),
            lambda g: g['call_contexts'][0].update(parent='missing'),
            lambda g: g['control_flow'][0].update(to='missing'),
            lambda g: g['obligations'][0].update(condition='missing'),
            lambda g: g['evidence'][0]['range'].update(end=99999),
            lambda g: g['evidence'][0].update(source_hash=HASH),
            lambda g: g.update(generation=g['generation']+1),
        ]
        for mutation in mutations:
            payload=self.payload();mutation(payload['graphs'][0])
            with self.assertRaises(ValueError): artifacts.envelope('state_graph',payload,[],HASH)

    def test_unsupported_version(self):
        import jsonschema
        payload=self.payload();payload['graphs'][0]['contract_version']=999
        with self.assertRaises(jsonschema.ValidationError): artifacts.envelope('state_graph',payload,[],HASH)

    def test_export_preserves_graph_and_trace(self):
        import task_runner
        from unittest.mock import patch
        payload=self.payload();graph=payload['graphs'][0]
        report={'documents':[{'state_graph':graph,'findings':[{'state_trace':t} for t in payload['traces']]}]}
        with patch.object(task_runner.lab,'read_source',side_effect=lambda p: '😀'.encode() if p.name == 't01-evidence.ts' else b'let d = 0; let result = 1 / d;'):
            result=task_runner.graph_payload(report,{'inputs':[{'path':'/tmp/t01-case.ts'},{'path':'/tmp/t01-evidence.ts'}]})
        self.assertEqual(result,payload)

    def test_typescript_contract_fixture(self):
        import json
        import subprocess
        import tempfile
        from pathlib import Path
        root=artifacts.SCHEMAS.parents[2]
        compiler=root/'extensions/vscode/node_modules/typescript/bin/tsc'
        if not compiler.exists(): self.skipTest('TypeScript dependencies are not installed')
        with tempfile.TemporaryDirectory() as directory:
            target=Path(directory)/'fixture.ts'
            contract=root/'extensions/vscode/src/state-contract'
            target.write_text('import { StateGraph, StateTrace } from '+json.dumps(str(contract))+';\nconst fixture: {graphs: StateGraph[]; traces: StateTrace[]} = '+json.dumps(self.payload())+';\n')
            subprocess.run(['node',str(compiler),'--noEmit','--strict','--skipLibCheck',str(target)],check=True,capture_output=True)
