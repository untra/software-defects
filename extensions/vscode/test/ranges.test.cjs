const { test } = require('node:test');
const assert = require('node:assert/strict');
const { segments, accepts } = require('../dist/ranges');
const r = (start, end) => ({ start: { line: start, character: 0 }, end: { line: end, character: 0 } });
const finding = (fn, origin = 'direct') => ({ functionRange: fn, triggerRange: r(1, 2), finding: { origin } });
test('a pure nested closure does not inherit its containing function color', () => {
  const outer = r(0, 10), inner = r(3, 6);
  const snapshot = { functions: [{ range: outer, body: outer }, { range: inner, body: r(4, 6) }], findings: [finding(outer)] };
  const result = segments(snapshot, false);
  assert.deepEqual(result.map(s => s.range), [r(0, 3), r(6, 10)]);
});
test('propagation can be hidden independently of direct signals', () => {
  const fn = r(0, 3), snapshot = { functions: [{ range: fn, body: fn }], findings: [finding(fn, 'propagated')] };
  assert.equal(segments(snapshot, false)[0].propagated, true);
  assert.deepEqual(segments(snapshot, true), []);
});
test('reject results after edits, workspace changes, or document switches', () => {
  const snapshot = { schemaVersion: 1, uri: 'file:///a.ts', version: 7 };
  assert.equal(accepts(snapshot, snapshot.uri, 7, 4, 4), true);
  assert.equal(accepts(snapshot, snapshot.uri, 8, 4, 4), false);
  assert.equal(accepts(snapshot, snapshot.uri, 7, 4, 5), false);
  assert.equal(accepts(snapshot, 'file:///b.ts', 7, 4, 4), false);
});
test('large trigger ranges exclude independently analyzed nested functions', () => {
  const { triggerSegments } = require('../dist/ranges');
  const outer = r(0, 12), inner = r(3, 6);
  const entry = { functionRange: outer, triggerRange: r(1, 9), finding: { origin: 'direct' } };
  const snapshot = { findings: [entry], functions: [{ range: outer, body: outer }, { range: inner, body: inner }] };
  assert.deepEqual(triggerSegments(snapshot, false).map(s => s.range), [r(1, 3), r(6, 9)]);
});
