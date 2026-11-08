"""Generate the strict interchange schemas from explicit artifact contracts."""
import json
import sys
import subprocess
import contract_types
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2] / 'state-lens' / 'schemas'
TEXT = {'type':'string', 'minLength':1}
HASH = {'type':'string', 'pattern':'^[0-9a-f]{64}$'}

def obj(properties):
    return {'type':'object','properties':properties,'required':list(properties),'additionalProperties':False}

def array(item):
    return {'type':'array','items':item}

def enum(*values):
    return {'enum':list(values)}

REF = obj({'artifact_id':HASH,'path':TEXT})
SOURCE = obj({'uri':TEXT,'source_hash':HASH,'byte_length':{'type':'integer','minimum':0}})
SPAN = obj({'uri':TEXT,'source_hash':HASH,'start':{'type':'integer','minimum':0},'end':{'type':'integer','minimum':0}})
PREDICATE = obj({'predicate':TEXT,'arguments':array(TEXT),'result':enum('established','refuted','unknown'),'evidence':array(SPAN)})
PAYLOADS = {
 'source_snapshot':obj({'sources':array(SOURCE)}),
 'state_graph':obj({'sources':array(SOURCE),'nodes':array(obj({'id':TEXT,'kind':TEXT,'location':SPAN})),'edges':array(obj({'from':TEXT,'to':TEXT,'kind':TEXT,'evidence':array(SPAN)})),'limitations':array(TEXT)}),
 'contract':obj({'property':TEXT,'applicability':array(PREDICATE),'provenance':array(REF),'review_status':enum('proposed','reviewed'),'reviewer':{'type':['string','null']}}),
 'alignment':obj({'matches':array(obj({'before':SPAN,'after':array(SPAN),'status':enum('unique','ambiguous','unmatched')})),'assumptions':array(TEXT)}),
 'semantic_delta':obj({'added':array(PREDICATE),'removed':array(PREDICATE),'changed':array(PREDICATE),'contract':REF,'failure_condition':TEXT,'blocking_postcondition':TEXT}),
 'rule':obj({'requires':array(PREDICATE),'breakers':array(PREDICATE),'requirement':TEXT,'limitations':array(TEXT),'contract':REF}),
 'compound':obj({'structure':enum('chain','composite'),'ingredients':{'type':'array','items':TEXT,'minItems':2},'relationships':array(PREDICATE),'ordering':array(PREDICATE),'breakers':array(PREDICATE),'outcome_property':TEXT,'cwes':array(TEXT)}),
 'finding':obj({'origins':array(SPAN),'invalidation_status':enum('invalidated','never_established','unknown'),'invalidations':array(SPAN),'affected_uses':array(SPAN),'supporting_spans':array(SPAN),'evidence_level':enum('candidate_relationship','feasible_under_model','reproduced'),'predicates':array(PREDICATE),'assumptions':array(TEXT),'witness':{'anyOf':[REF,{'type':'null'}]},'exploitability':obj({'status':enum('unknown','supported','refuted'),'actor_control':array(PREDICATE),'reachability':array(PREDICATE),'boundary':{'type':['string','null']},'security_property':{'type':['string','null']}})}),
 'repair':obj({'preconditions':array(PREDICATE),'transformation':TEXT,'postconditions':array(PREDICATE),'preservation':array(TEXT),'contract':REF}),
 'evaluation':obj({'split':enum('development','validation','held_out'),'groups':array(TEXT),'checks':array(PREDICATE),'outcome':enum('passed','failed','unsupported'),'reproduction':{'anyOf':[REF,{'type':'null'}]}}),
 'hypothesis':obj({'claim':TEXT,'required_structure':array(PREDICATE),'contract':REF,'counterexamples':array(TEXT),'assumptions':array(TEXT)}),
 'challenge':obj({'hypothesis':REF,'cases':array(obj({'description':TEXT,'expected_property':TEXT,'variation':enum('reassignment','alias','early_return','nested_scope','exception','schedule','benign')}))}),
 'tool_result':obj({'tool':TEXT,'exit_code':{'type':'integer'},'stdout':{'type':'string'},'stderr':{'type':'string'}}),
}

def emit(path, content):
    if path.suffix == '.rs':
        content = subprocess.run(['rustfmt', '--edition', '2021'], input=content, text=True, capture_output=True, check=True).stdout
    if '--check' in sys.argv:
        if not path.exists() or path.read_text() != content:
            raise SystemExit(f'Generated contract is stale: {path}')
    else:
        path.write_text(content)

def main():
    for path, content in contract_types.generated().items():
        if path.suffix == '.ts': content += contract_types.wire_types()
        emit(path, content)
    PAYLOADS['state_graph'] = obj({'graphs':array({'$ref':'#/$defs/StateGraph'}),'traces':array({'$ref':'#/$defs/StateTrace'})})
    ROOT.mkdir(parents=True,exist_ok=True)
    for kind,payload in PAYLOADS.items():
        schema = obj({'schema_version':{'const':2},'artifact_id':HASH,'kind':{'const':kind},'producer':obj({'name':TEXT,'version':TEXT}),'inputs':array({'oneOf':[obj({'kind':{'const':'file'},'path':TEXT,'byte_digest':HASH}),obj({'kind':{'const':'artifact'},'path':TEXT,'byte_digest':HASH,'artifact_id':HASH})]}),'configuration_hash':HASH,'completeness':enum('complete','partial','unsupported'),'payload':payload})
        schema.update({'$schema':'https://json-schema.org/draft/2020-12/schema','$id':f'https://state-lens.local/schemas/{kind}.schema.json'})
        if kind == 'state_graph': schema['$defs'] = contract_types.wire_definitions()
        emit(ROOT/f'{kind}.schema.json',json.dumps(schema,indent=2)+'\n')
    packet = obj({'schema_version':{'const':1},'stage':enum('inventory','extract','align','compare','hypothesize','challenge','evaluate','repair','review','held_out'),'inputs':array(obj({'path':TEXT,'sha256':HASH})),'tool':enum('inventory','scan','graph','semantic_facts','semantic_compare','semantic_preview','submission'),'case':{'type':['string','null']},'source':{'type':['string','null']},'output_kind':enum(*PAYLOADS),'behavioral_execution':{'type':'boolean'},'split':enum('development','validation','held_out'),'groups':{'type':'array','items':TEXT,'minItems':1},'configuration_hash':HASH,'limits':obj({'timeout_seconds':{'type':'integer','minimum':1,'maximum':600},'output_bytes':{'type':'integer','minimum':1,'maximum':16777216}})})
    packet['$schema']='https://json-schema.org/draft/2020-12/schema'
    emit(ROOT/'task.schema.json',json.dumps(packet,indent=2)+'\n')

if __name__=='__main__':main()
