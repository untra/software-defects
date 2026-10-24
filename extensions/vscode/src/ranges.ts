export interface Position { line: number; character: number }
export interface Range { start: Position; end: Position }
export interface FunctionRange { range: Range; body: Range; name: string }
export interface Evidence { uri: string; range: { start: number; end: number }; rule_id: string }
export interface Finding {
  rule_id: string;
  defect: string;
  family: string;
  effect: { kind: string; parameter?: number };
  origin: 'direct' | 'propagated';
  resolution: string;
  explanation: string;
  evidence: Evidence[];
  call_path: Evidence[];
  related_cwes?: string[];
}
export interface Entry { finding: Finding; triggerRange: Range; functionRange: Range | null }
export interface Snapshot {
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
