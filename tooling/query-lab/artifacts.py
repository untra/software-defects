"""Strict artifact boundaries and content-addressed provenance."""
import hashlib
import json
from pathlib import Path
import jsonschema

SCHEMAS = Path(__file__).resolve().parents[1] / 'state-lens' / 'schemas'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()

def identity(value):
    return digest(canonical({k:v for k,v in value.items() if k!='artifact_id'}))

def validate(value, kind=None):
    kind = kind or value.get('kind')
    allowed = {p.name.removesuffix('.schema.json') for p in SCHEMAS.glob('*.schema.json')}
    if kind not in allowed: raise ValueError('unknown artifact kind')
    version = value.get('schema_version')
    if version not in (1, 2) or (kind == 'task' and version != 1): raise ValueError('unsupported schema version')
    directory = SCHEMAS / 'v1' if version == 1 and kind != 'task' else SCHEMAS
    schema = json.loads((directory/f'{kind}.schema.json').read_text())
    jsonschema.Draft202012Validator(schema).validate(value)
    if kind == 'task': return
    if value['artifact_id'] != identity(value): raise ValueError('artifact content hash mismatch')
    payload = value['payload']
    def spans(node):
        if isinstance(node,dict):
            if {'uri','source_hash','start','end'} <= node.keys():
                if node['end'] < node['start']: raise ValueError('reversed source span')
            for child in node.values(): spans(child)
        elif isinstance(node,list):
            for child in node: spans(child)
    spans(payload)
    if kind == 'state_graph' and version == 2:
        validate_graphs(payload)
    if kind == 'state_graph' and version == 1:
        nodes = {n['id']:n for n in payload['nodes']}
        if len(nodes)!=len(payload['nodes']): raise ValueError('duplicate graph identity')
        sources = {(s['uri'],s['source_hash']):s for s in payload['sources']}
        locations = [n['location'] for n in nodes.values()]
        for edge in payload['edges']:
            if edge['from'] not in nodes or edge['to'] not in nodes: raise ValueError('dangling graph edge')
            locations.extend(edge['evidence'])
        for loc in locations:
            source=sources.get((loc['uri'],loc['source_hash']))
            if source is None or loc['end']>source['byte_length']: raise ValueError('stale or invalid graph span')
    if kind == 'finding':
        if payload['invalidation_status']!='invalidated' and payload['invalidations']: raise ValueError('invented invalidation range')
        if payload['evidence_level']!='candidate_relationship':
            if payload['witness'] is None or any(p['result']=='unknown' for p in payload['predicates']): raise ValueError('unsupported evidence promotion')
    if kind == 'contract' and payload['review_status']=='reviewed' and not payload['reviewer']:
        raise ValueError('reviewed contract requires reviewer provenance')

def envelope(kind,payload,inputs,configuration_hash,completeness='complete',version=2):
    value={'schema_version':version,'kind':kind,'producer':{'name':'query-lab','version':'1'},'inputs':inputs,'configuration_hash':configuration_hash,'completeness':completeness,'payload':payload}
    value['artifact_id']=identity(value)
    validate(value)
    return value


def validate_graphs(payload):
    graphs = payload['graphs']
    if not graphs: raise ValueError('missing graph')
    first = graphs[0]
    documents = {d['id']: d for d in first['documents']}
    if len(documents) != len(first['documents']) or len({d['uri'] for d in documents.values()}) != len(documents):
        raise ValueError('duplicate document identity')
    if first['documents'] != sorted(first['documents'], key=lambda d:d['id']): raise ValueError('noncanonical document order')
    snapshot = first['snapshot_id']
    manifest = [first['generation'], first['documents']]
    expected = digest(canonical(manifest))
    if snapshot != expected: raise ValueError('snapshot identity mismatch')
    tables = {k:{} for k in ('operations','bindings','allocations','blocks','conditions','access_paths','call_contexts','evidence','obligations')}
    all_ids = set()
    for graph in graphs:
        if graph['snapshot_id'] != snapshot or graph['generation'] != first['generation'] or graph['documents'] != first['documents']:
            raise ValueError('mixed snapshots')
        for kind, table in tables.items():
            for item in graph[kind]:
                key = item['identity' if kind == 'operations' else 'id']
                if key in all_ids: raise ValueError('duplicate graph identity')
                if not key.startswith(snapshot + ':'): raise ValueError('mixed snapshot identity')
                all_ids.add(key); table[key] = item
    def ref(key, *kinds):
        if key is not None and not any(key in tables[k] for k in kinds):
            raise ValueError('dangling or wrong-kind reference: '+key)
    def refs(keys, *kinds):
        for key in keys: ref(key, *kinds)
    def span(document, span, source_hash=None):
        doc = documents.get(document)
        if doc is None or (source_hash is not None and doc['source_hash'] != source_hash):
            raise ValueError('stale source span')
        if not 0 <= span['start'] <= span['end'] <= doc['byte_length']: raise ValueError('invalid source span')
    def location(loc):
        matches = [d for d in documents.values() if d['uri'] == loc['uri']]
        if len(matches) != 1: raise ValueError('unknown source span')
        span(matches[0]['id'], loc['range'])
    def local_edges(container):
        ops={op['id']:op for op in container['operations']}
        if len(ops) != len(container['operations']): raise ValueError('duplicate local operation identity')
        for op in ops.values():
            for guard in op['guards']:
                if guard not in ops or ops[guard]['kind'] != 'guard': raise ValueError('dangling guard')
        for edge in container['edges']:
            if edge['from'] not in ops or edge['to'] not in ops: raise ValueError('dangling dependency edge')
    for graph in graphs:
        local_edges(graph)
        for op in graph['operations']:
            location(op['location'])
            doc = next(d for d in documents.values() if d['uri'] == op['location']['uri'])
            if graph['source_hash'] not in {d['source_hash'] for d in documents.values()}: raise ValueError('mixed source snapshot')
            if op['binding'] is not None: span(doc['id'], op['binding'])
            for field, kind in [('binding_id','bindings'),('allocation_id','allocations'),('block_id','blocks'),('access_path','access_paths'),('call_context','call_contexts')]: ref(op[field], kind)
            refs(op['required_conditions'], 'conditions'); refs(op['evidence'], 'evidence')
        for edge in graph['control_flow']:
            ref(edge['from'],'blocks'); ref(edge['to'],'blocks'); ref(edge['condition'],'conditions'); refs(edge['evidence'],'evidence')
    for item in tables['bindings'].values(): location(item['location'])
    for item in tables['allocations'].values(): ref(item['operation'],'operations')
    for item in tables['blocks'].values(): refs(item['operations'],'operations')
    for item in tables['evidence'].values(): span(item['document'],item['range'],item['source_hash'])
    for item in tables['call_contexts'].values():
        ref(item['parent'],'call_contexts'); ref(item['call'],'operations')
        seen={item['id']}; parent=item['parent']
        while parent is not None:
            if parent in seen: raise ValueError('cyclic call context')
            seen.add(parent); parent=tables['call_contexts'][parent]['parent']
    for item in tables['access_paths'].values():
        ref(item['root'],'bindings','allocations')
        for segment in item['segments']:
            if segment['kind']=='index': ref(segment['value'],'operations','access_paths')
    for item in tables['conditions'].values():
        refs(item['evidence'],'evidence')
        predicate=item['predicate']
        if predicate['kind'] in ('conjunction','disjunction'): refs(predicate['conditions'],'conditions')
        else:
            for key in ('value','index','collection','resource','owner','location','left','right'):
                if key in predicate: ref(predicate[key],'operations','access_paths','bindings','allocations')
    for item in tables['obligations'].values():
        ref(item['operation'],'operations'); ref(item['condition'],'conditions'); refs(item['evidence'],'evidence')
        if item['outcome']=='violated_on_demonstrated_path' and not item['evidence']: raise ValueError('demonstrated violation requires evidence')
    for trace in payload['traces']:
        if trace['snapshot_id'] != snapshot or trace['source_hash'] not in {g['source_hash'] for g in graphs}: raise ValueError('mixed trace snapshot')
        for field in ('origin_operations','invalidating_operations','affected_operations'): refs(trace[field],'operations')
        refs(trace['supporting_evidence'],'evidence'); refs(trace['required_conditions'],'conditions')
        if trace['invalidation_status'] != 'invalidated' and (trace['invalidating_operations'] or trace['invalidations']): raise ValueError('invented invalidation')
        local_edges(trace)
        for op in trace['operations']:
            if tables['operations'].get(op['identity']) != op: raise ValueError('inconsistent trace operation')
        for edge in trace['edges']:
            candidates = [g for g in graphs if any(op['identity'] in {t['identity'] for t in trace['operations']} for op in g['operations'])]
            if not any(edge in g['edges'] for g in candidates): raise ValueError('inconsistent trace edge')
        for field in ('origins','invalidations','affected_uses'):
            for loc in trace[field]: location(loc)
