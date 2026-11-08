"""Execute registered tools or validate model submissions from immutable task packets."""
import argparse
import json
from pathlib import Path
import sys
import tempfile
import jsonschema

import artifacts
import query_lab as lab

STAGES = ('inventory','extract','align','compare','hypothesize','challenge','evaluate','repair','review','held_out')
STAGE_TOOLS = {
 'inventory':{'inventory'}, 'extract':{'scan','semantic_facts','graph'},
 'align':{'submission'}, 'compare':{'semantic_compare','submission'},
 'hypothesize':{'submission'}, 'challenge':{'submission'},
 'evaluate':{'submission'}, 'repair':{'semantic_preview','submission'},
 'review':{'submission'}, 'held_out':{'submission'},
}
STAGE_OUTPUTS = {'align':{'alignment'},'compare':{'semantic_delta','tool_result'},'hypothesize':{'hypothesis'},'challenge':{'challenge'},'evaluate':{'evaluation'},'repair':{'repair','tool_result'},'review':{'evaluation'},'held_out':{'evaluation'}}


def read(path):
    return json.loads(lab.read_source(path))


def configuration(analyzer):
    return artifacts.digest(artifacts.canonical({
        'analyzer':artifacts.digest(analyzer.read_bytes()),
        'runner':artifacts.digest(Path(__file__).read_bytes()),
        'lab':artifacts.digest(Path(lab.__file__).read_bytes()),
        'validator':artifacts.digest(Path(artifacts.__file__).read_bytes()),
        'helpers':{p.name:artifacts.digest(p.read_bytes()) for p in lab.HERE.glob('*.py')},
        'corpus':{str(p.relative_to(lab.ROOT)):artifacts.digest(p.read_bytes()) for root in ('examples','examples-fixed','tooling/query-lab/cases') for p in sorted((lab.ROOT/root).rglob('*')) if p.is_file()},
        'schemas':{p.name:artifacts.digest(p.read_bytes()) for p in sorted(artifacts.SCHEMAS.glob('*.json'))},
    }))


def check_packet(packet, analyzer):
    artifacts.validate(packet,'task')
    if packet['tool'] not in STAGE_TOOLS[packet['stage']]: raise ValueError('tool not allowed for stage')
    if packet['stage'] in STAGE_OUTPUTS and packet['output_kind'] not in STAGE_OUTPUTS[packet['stage']]: raise ValueError('wrong stage output kind')
    if packet['configuration_hash'] != configuration(analyzer): raise ValueError('stale task configuration')
    if packet['stage']=='held_out' and packet['split']!='held_out': raise ValueError('held-out evaluation requires an independently frozen split')
    if packet['stage'] in {'align','compare','hypothesize','challenge','repair'} and packet['split']=='held_out': raise ValueError('held-out data cannot enter rule development')
    inputs=[]
    for item in packet['inputs']:
        path=Path(item['path']).resolve()
        data=lab.read_source(path)
        if artifacts.digest(data)!=item['sha256']: raise ValueError(f'stale input: {path}')
        entry = {'kind':'file','byte_digest':item['sha256'],'path':str(path)}
        try:
            candidate = json.loads(data)
        except (ValueError, UnicodeError):
            candidate = None
        if isinstance(candidate, dict) and 'artifact_id' in candidate:
            artifacts.validate(candidate)
            entry.update(kind='artifact', artifact_id=candidate['artifact_id'])
        inputs.append(entry)
    if packet['source'] and str(Path(packet['source']).resolve()) not in {i['path'] for i in inputs}:
        raise ValueError('source must be a declared input')
    if packet['tool'] in {'semantic_compare','semantic_preview'}:
        cases=[c for c in lab.corpus(include_unsupported=True) if c['id']==packet['case']]
        if len(cases)!=1: raise ValueError('unknown case')
        declared={i['path'] for i in inputs}
        if any(str((lab.ROOT/cases[0][k]).resolve()) not in declared for k in ('original','fixed','sidecar')):
            raise ValueError('case sources and sidecar must be declared inputs')
    return inputs


def commands(packet,analyzer):
    tool=packet['tool']
    if tool=='inventory': return [sys.executable,lab.HERE/'query_lab.py','semantic-inventory']
    if tool in {'scan','semantic_facts','graph'}:
        if not packet['source']: raise ValueError('source required')
        return [analyzer,'semantic-facts' if tool=='semantic_facts' else 'scan',Path(packet['source']).resolve()]
    if tool in {'semantic_compare','semantic_preview'}:
        if not packet['case']: raise ValueError('case required')
        command=[sys.executable,lab.HERE/'query_lab.py',tool.replace('_','-'),'--case',packet['case'],'--analyzer',analyzer]
        if packet['behavioral_execution']: command.append('--run-behavior')
        return command
    raise ValueError('no registered tool')


def run(packet,analyzer,out,submission=None,allow_behavior=False,resume=False):
    inputs=check_packet(packet,analyzer)
    if packet['behavioral_execution'] and not allow_behavior: raise ValueError('behavioral execution requires --allow-behavior')
    key=artifacts.digest(artifacts.canonical(packet))
    destination=out/f'{key}.json'
    if resume and destination.exists() and packet['tool']!='submission':
        result=read(destination)
        artifacts.validate(result)
        if result['inputs']!=inputs or result['configuration_hash']!=packet['configuration_hash']: raise ValueError('stale saved result')
        return result
    if packet['tool']=='submission':
        if submission is None: raise ValueError('stage awaits a model/manual submission')
        value=read(submission)
        if set(value)!={'kind','payload'} or value['kind']!=packet['output_kind']: raise ValueError('unexpected model output')
        if value['kind']=='contract' and value['payload'].get('review_status')=='reviewed': raise ValueError('models cannot self-review contracts')
        result=artifacts.envelope(value['kind'],value['payload'],inputs,packet['configuration_hash'],'partial')
    else:
        if packet['output_kind']!=('state_graph' if packet['tool']=='graph' else 'tool_result'): raise ValueError('wrong tool output kind')
        code,stdout,stderr=lab.execute(commands(packet,analyzer),timeout=packet['limits']['timeout_seconds'])
        if len(stdout.encode())+len(stderr.encode())>packet['limits']['output_bytes']: raise ValueError('task output budget exceeded')
        if packet['tool']=='graph':
            if code: raise ValueError('graph extraction failed: '+stderr)
            report=json.loads(stdout)
            result=artifacts.envelope('state_graph',graph_payload(report,packet),inputs,packet['configuration_hash'],'partial')
        else:
            result=artifacts.envelope('tool_result',{'tool':packet['tool'],'exit_code':code,'stdout':stdout,'stderr':stderr},inputs,packet['configuration_hash'],'complete' if code==0 else 'partial')
    validate_links(result)
    out.mkdir(parents=True,exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w',dir=out,delete=False) as temporary:
        json.dump(result,temporary,indent=2)
        temporary.write('\n')
        name=temporary.name
    Path(name).replace(destination)
    return result


def plan(case,analyzer):
    selected=[c for c in lab.corpus(include_unsupported=True) if c['id']==case]
    if len(selected)!=1: raise ValueError('case must identify one corpus pair')
    item=selected[0]
    paths=[(lab.ROOT/item[k]).resolve() for k in ('original','fixed','sidecar')]
    inputs=[{'path':str(p),'sha256':artifacts.digest(lab.read_source(p))} for p in paths]
    stages=[]
    for stage in STAGES:
        tool={'inventory':'inventory','extract':'graph','compare':'semantic_compare','repair':'semantic_preview'}.get(stage,'submission')
        output={'extract':'state_graph','align':'alignment','hypothesize':'hypothesis','challenge':'challenge','evaluate':'evaluation','review':'evaluation','held_out':'evaluation'}.get(stage,'tool_result')
        packet={'schema_version':1,'stage':stage,'inputs':inputs,'tool':tool,'case':case,'source':str(paths[0]) if stage=='extract' else None,'output_kind':output,'behavioral_execution':False,'split':'development','groups':[item['family']+'/'+item['defect']],'configuration_hash':configuration(analyzer),'limits':{'timeout_seconds':lab.COMMAND_TIMEOUT,'output_bytes':lab.MAX_OUTPUT_BYTES}}
        stages.append({'packet':packet,'depends_on':list(STAGES[:STAGES.index(stage)])})
    return {'schema_version':1,'stages':stages,'limitations':['Add verified project/family/translation groups before held-out evaluation.','Model stages require validated submissions; no model provider is implicitly invoked.']}


def graph_payload(report,packet):
    from urllib.parse import urlparse, unquote
    graphs=[]; traces=[]
    declared = {str(Path(i['path']).resolve()) for i in packet['inputs']}
    for document in report['documents']:
        graph=document.get('state_graph')
        if graph is None: raise ValueError('analyzer does not export production graph')
        for source in graph['documents']:
            uri=urlparse(source['uri'])
            if uri.scheme != 'file' or uri.netloc not in ('', 'localhost'):
                raise ValueError('graph source is not a local file')
            path=Path(unquote(uri.path)).resolve()
            if str(path) not in declared: raise ValueError('undeclared graph source')
            data=lab.read_source(path)
            if artifacts.digest(data)!=source['source_hash'] or len(data)!=source['byte_length']:
                raise ValueError('graph source changed during analysis')
        graphs.append(graph)
        traces.extend(f['state_trace'] for f in document['findings'] if f.get('state_trace'))
    return {'graphs':graphs,'traces':traces}


def validate_links(value):
    declared={str(Path(i['path']).resolve()) for i in value['inputs']}
    def visit(node):
        if isinstance(node,dict):
            if set(node)=={'artifact_id','path'}:
                path=str(Path(node['path']).resolve())
                if path not in declared: raise ValueError('undeclared artifact reference')
                target=read(path)
                artifacts.validate(target)
                if target['artifact_id']!=node['artifact_id']: raise ValueError('stale artifact reference')
            for child in node.values(): visit(child)
        elif isinstance(node,list):
            for child in node: visit(child)
    visit(value['payload'])


def chain(plan,analyzer,out,submissions,allow_behavior=False):
    if out is None: raise ValueError('--output required')
    completed={}
    results=[]
    for item in plan['stages']:
        packet=json.loads(json.dumps(item['packet']))
        if packet['stage'] in completed: raise ValueError('duplicate chain stage')
        for dependency in item['depends_on']:
            if dependency not in completed: raise ValueError('missing or unordered dependency')
            path=completed[dependency]
            packet['inputs'].append({'path':str(path.resolve()),'sha256':artifacts.digest(path.read_bytes())})
        submission=submissions/f"{packet['stage']}.json" if submissions else None
        if packet['tool']=='submission' and (submission is None or not submission.exists()):
            return {'status':'awaiting_submission','stage':packet['stage'],'packet':packet,'results':results}
        result=run(packet,analyzer,out,submission,allow_behavior,resume=True)
        results.append(result)
        key=artifacts.digest(artifacts.canonical(packet))
        completed[packet['stage']]=out/f'{key}.json'
        if result['kind']=='tool_result' and result['payload']['exit_code']!=0:
            return {'status':'failed','stage':packet['stage'],'results':results}
    return {'status':'complete','results':results}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['plan','run','resume','validate','chain'])
    parser.add_argument('--analyzer',type=Path)
    parser.add_argument('--case')
    parser.add_argument('--packet',type=Path)
    parser.add_argument('--submission',type=Path)
    parser.add_argument('--output',type=Path)
    parser.add_argument('--allow-behavior',action='store_true')
    args=parser.parse_args()
    try:
        if args.command=='validate':
            value=read(args.packet); artifacts.validate(value,'task' if 'stage' in value else None); result={'valid':True}
        elif args.command=='chain':
            result=chain(read(args.packet),args.analyzer.resolve(),args.output,args.submission,args.allow_behavior)
        elif args.command=='plan': result=plan(args.case,args.analyzer.resolve())
        else:
            if args.output is None: raise ValueError('--output is required')
            result=run(read(args.packet),args.analyzer.resolve(),args.output,args.submission,args.allow_behavior,args.command=='resume')
        print(json.dumps(result,indent=2)); return 0
    except (ValueError,OSError,KeyError,lab.LabError,jsonschema.ValidationError) as error:
        print(json.dumps({'error':str(error)})); return lab.EXIT_ERROR

if __name__=='__main__':sys.exit(main())
