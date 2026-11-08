export interface Position { line: number; character: number }
export interface Range { start: Position; end: Position }
export interface FunctionRange { range: Range; body: Range; name: string }
export interface Evidence { uri: string; range: { start: number; end: number }; rule_id: string; role?: string; source_hash?: string }
export interface TraceLocation { uri: string; range: { start: number; end: number } }
export type StateTrace = Pick<import('./state-contract').StateTrace,
  'source_hash' | 'evidence_level' | 'origins' | 'invalidations' | 'affected_uses' | 'required_condition' | 'assumptions'>
  & Partial<import('./state-contract').StateTrace>;

export interface Finding {
  state_trace?: StateTrace;
  division_risk?: { confidence: string; outcome: string; mutated: boolean };
  rule_id: string;
  trigger?: { start: number; end: number };
  defect: string;
  family: string;
  compound?: { class: string; category: string; ingredients: [Evidence, Evidence] };
  effect: { kind: string; parameter?: number };
  origin: 'direct' | 'propagated';
  resolution: string;
  explanation: string;
  evidence: Evidence[];
  call_path: Evidence[];
  related_cwes?: string[];
}
export interface Entry { finding: Finding; triggerRange: Range; blastRadius?: Range[]; functionRange: Range | null }
export interface StateAnnotation {
  depth?: number;
  category: 'compound-defects' | 'mutated-state' | 'unstructured-state' | 'shared-state';
  target: 'function' | 'property' | 'segment';
  range: Range;
  position: Position;
  label: 'Untyped' | 'Unstable' | 'Fragile' | 'Corruptible' | 'Impure' | 'Mutacious' | 'Volatile' | 'Polymorphic' | 'Unstructured' | 'Insecure' | 'Async' | 'Shared' | 'Lifecycle';
  functionRange: Range | null;
  findings: number[];
}
export function annotations(snapshot: Snapshot, directOnly: boolean): StateAnnotation[] {
  const groups = new Map<string, StateAnnotation[]>();
  for (const annotation of snapshot.annotations ?? []) {
    const findings = annotation.findings.filter(index => snapshot.findings[index] && (!directOnly || snapshot.findings[index].finding.origin === 'direct'));
    if (!findings.length) continue;
    const key = `${annotation.category}:${annotation.position.line}:${annotation.category === 'compound-defects' ? annotation.label : ''}`;
    groups.set(key, [...(groups.get(key) ?? []), { ...annotation, findings }]);
  }
  return [...groups.values()].map(group => {
    group.sort((a, b) => (b.depth ?? 0) - (a.depth ?? 0) || (a.range.end.line - a.range.start.line) - (b.range.end.line - b.range.start.line) ||
      (a.range.end.character - a.range.start.character) - (b.range.end.character - b.range.start.character) || compare(b.position, a.position));
    const selected = { ...group[0], findings: [...new Set(group.flatMap(a => a.findings))] };
    if (group.length > 1 && selected.category !== 'compound-defects') {
      selected.target = 'segment';
      selected.label = selected.category === 'mutated-state' ? 'Mutacious' : selected.category === 'unstructured-state' ? 'Unstructured' : 'Shared';
    }
    return selected;
  });
}
export interface Snapshot {
  annotations?: StateAnnotation[];
  schemaVersion: number;
  generation: number;
  uri: string;
  version: number;
  sourceHash: string;
  findings: Entry[];
  functions: FunctionRange[];
  unresolved: { reason: string }[];
  parseErrors: boolean;
  indexing?: boolean;
}
export interface Segment { range: Range; propagated: boolean; entries: Entry[] }
export function compare(a: Position, b: Position): number { return a.line - b.line || a.character - b.character; }
export function contains(a: Range, b: Range): boolean { return compare(a.start, b.start) <= 0 && compare(a.end, b.end) >= 0; }
export function equal(a: Range, b: Range): boolean { return compare(a.start, b.start) === 0 && compare(a.end, b.end) === 0; }

export function segments(snapshot: Snapshot, directOnly: boolean): Segment[] {
  const entries = snapshot.findings.filter(e => !directOnly || e.finding.origin === 'direct');
  const points = snapshot.functions.flatMap(f => [f.body.start, f.body.end, f.range.start, f.range.end]);
  points.sort(compare);
  const unique = points.filter((p, i) => i === 0 || compare(points[i - 1], p) !== 0);
  const result: Segment[] = [];
  for (let i = 0; i + 1 < unique.length; i++) {
    const range = { start: unique[i], end: unique[i + 1] };
    const scopes = snapshot.functions.filter(f => contains(f.range, range));
    const inner = scopes.find(f => !scopes.some(other => !equal(f.range, other.range) && contains(f.range, other.range)));
    if (!inner || !contains(inner.body, range)) continue;
    const active = entries.filter(e => e.functionRange && equal(e.functionRange, inner.range));
    if (active.length) result.push({ range, propagated: active.every(e => e.finding.origin === 'propagated'), entries: active });
  }
  return result;
}
export function accepts(snapshot: Snapshot, uri: string, version: number, requestedEpoch: number, currentEpoch: number): boolean {
  return snapshot.schemaVersion === 1 && snapshot.uri === uri && snapshot.version === version && requestedEpoch === currentEpoch;
}

export function triggerSegments(snapshot: Snapshot, directOnly: boolean): { range: Range; entry: Entry }[] {
  return snapshot.findings.filter(e => !directOnly || e.finding.origin === 'direct').flatMap(entry => {
    const excluded = snapshot.functions.filter(f => contains(entry.triggerRange, f.range) && (!entry.functionRange || !equal(entry.functionRange, f.range))).map(f => f.range);
    const points = [entry.triggerRange.start, entry.triggerRange.end, ...excluded.flatMap(r => [r.start, r.end])].sort(compare);
    const unique = points.filter((p, i) => i === 0 || compare(points[i - 1], p) !== 0);
    return unique.slice(0, -1).map((start, i) => ({ start, end: unique[i + 1] })).filter(r => !excluded.some(e => contains(e, r))).map(range => ({ range, entry }));
  });
}

export const PALETTE = {
  'compound-defects': '#C9A227',
  'mutated-state': '#78A65A',
  'unstructured-state': '#3D85C6',
  'shared-state': '#DD3333',
  'division-risk': '#FFE599',
  nondeterminism: '#C9A227',
  'runtime-errors': '#C9A227',
  'data-corruption': '#C9A227'
} as const;
export type Category = keyof typeof PALETTE;
const FAMILY_ORDER: Category[] = ['mutated-state', 'unstructured-state', 'shared-state'];
export interface Decoration {
  category: Category;
  range: Range;
  propagated: boolean;
  outline: boolean;
  entries: Entry[];
  families: string[];
}
function category(entry: Entry): Category | undefined {
  const key = entry.finding.compound?.class ?? entry.finding.family;
  return Object.prototype.hasOwnProperty.call(PALETTE, key) ? key as Category : undefined;
}
export function decorations(snapshot: Snapshot, directOnly: boolean): Decoration[] {

  const base = snapshot.findings.filter(entry => (!directOnly || entry.finding.origin === 'direct') && !entry.finding.compound && category(entry)).flatMap(entry => (entry.blastRadius ?? []).map(range => ({ entry, range })));
  const points = base.flatMap(p => [p.range.start, p.range.end]).sort(compare);
  const unique = points.filter((p, i) => i === 0 || compare(points[i - 1], p) !== 0);
  const result: Decoration[] = [];
  for (let i = 0; i + 1 < unique.length; i++) {
    const range = { start: unique[i], end: unique[i + 1] };
    const active = base.filter(p => contains(p.range, range)).map(p => p.entry);
    active.sort((a, b) => {
      if (a.finding.trigger && b.finding.trigger) {
        const width = (a.finding.trigger.end - a.finding.trigger.start) - (b.finding.trigger.end - b.finding.trigger.start);
        if (width) return width;
      }
      if (!equal(a.triggerRange, b.triggerRange)) {
        if (contains(a.triggerRange, b.triggerRange)) return 1;
        if (contains(b.triggerRange, a.triggerRange)) return -1;
        const lines = (a.triggerRange.end.line - a.triggerRange.start.line) - (b.triggerRange.end.line - b.triggerRange.start.line);
        const columns = (a.triggerRange.end.character - a.triggerRange.start.character) - (b.triggerRange.end.character - b.triggerRange.start.character);
        if (lines || columns) return lines || columns;
      }
      return FAMILY_ORDER.indexOf(category(a)!) - FAMILY_ORDER.indexOf(category(b)!) || a.finding.rule_id.localeCompare(b.finding.rule_id);
    });
    if (!active.length) continue;
    const families = FAMILY_ORDER.filter(f => active.some(e => e.finding.family === f));
    result.push({ range, category: category(active[0])!, propagated: active[0].finding.origin === 'propagated', outline: false, entries: active, families });
  }
  for (const { entry, range } of snapshot.findings.filter(entry => entry.finding.compound && (!directOnly || entry.finding.origin === 'direct')).flatMap(entry => (entry.blastRadius ?? []).map(range => ({ entry, range })))) {
    const key = category(entry);
    if (key) result.push({ range, category: key, propagated: entry.finding.origin === 'propagated', outline: true, entries: [entry], families: [] });
  }
  return result;
}

export function functionMarkers(snapshot: Snapshot, directOnly: boolean): { range: Range; category: Category; label: string; entries: Entry[] }[] {
  return annotations(snapshot, directOnly).filter(annotation => annotation.target === 'function').map(annotation => ({ range: annotation.range, category: annotation.category, label: annotation.label, entries: annotation.findings.map(index => snapshot.findings[index]).filter(entry => !directOnly || entry.finding.origin === 'direct') }));
}

export const CATEGORY_STYLE: Record<Category, 'fill' | 'outline'> = {
  'mutated-state': 'fill', 'unstructured-state': 'fill', 'shared-state': 'fill',
  'compound-defects': 'outline', 'division-risk': 'fill', nondeterminism: 'outline', 'runtime-errors': 'outline', 'data-corruption': 'outline'
};
export function divisionCompatible(capabilities: { experimental?: unknown } | undefined): boolean {
  const experimental = capabilities?.experimental as { stateLens?: { divisionAnalysis?: number } } | undefined;
  return experimental?.stateLens?.divisionAnalysis === 1;
}

export function findingEvidence(finding: Finding): Evidence[] {
  const result = [...finding.call_path, ...finding.evidence, ...(finding.compound?.ingredients ?? [])];
  const trace = finding.state_trace;
  if (trace) {
    for (const [role, locations] of [['Origin', trace.origins], ['Invalidation', trace.invalidations], ['Affected use', trace.affected_uses]] as const) {
      result.push(...locations.map(location => ({ ...location, rule_id: finding.rule_id, role, source_hash: trace.source_hash })));
    }
  }
  return result;
}
