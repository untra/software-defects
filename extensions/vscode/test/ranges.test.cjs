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

test('overlapping families split fills, preserve hovers, and paint a separate compound outline', () => {
  const { decorations } = require('../dist/ranges');
  const entry = (family, start, end, compound) => ({ triggerRange: r(start, end), functionRange: null,
    finding: { family, rule_id: family, origin: 'direct', compound } });
  const snapshot = { functions: [], findings: [entry('shared-state', 0, 6), entry('mutated-state', 2, 4),
    entry('compound-defects', 2, 5, { class: 'nondeterminism' })] };
  const result = decorations(snapshot, false);
  assert.deepEqual(result.filter(d => !d.outline).map(d => [d.category, d.range]), [
    ['shared-state', r(0, 2)], ['mutated-state', r(2, 4)], ['shared-state', r(4, 6)]
  ]);
  assert.equal(result[1].entries.length, 2);
  assert.deepEqual(result[1].families, ['mutated-state', 'shared-state']);
  assert.equal(result.filter(d => d.outline)[0].category, 'nondeterminism');
  assert.deepEqual(decorations({ ...snapshot, findings: [] }, false), []);
});

test('equal trigger widths use a stable family order independent of finding order', () => {
  const { decorations } = require('../dist/ranges');
  const findings = ['shared-state', 'unstructured-state', 'mutated-state'].map(family => ({
    triggerRange: r(1, 2), functionRange: null, finding: { family, rule_id: family, origin: 'direct' }
  }));
  const snapshot = { functions: [], findings };
  assert.equal(decorations(snapshot, false)[0].category, 'mutated-state');
  assert.equal(decorations({ ...snapshot, findings: [...findings].reverse() }, false)[0].category, 'mutated-state');
});

test('decoration palette matches authoritative website CSS', () => {
  const fs = require('node:fs');
  const path = require('node:path');
  const { PALETTE } = require('../dist/ranges');
  const css = fs.readFileSync(path.resolve(__dirname, '../../../src/assets/css/style.css'), 'utf8');
  const variables = { 'mutated-state': 'mutated', 'unstructured-state': 'unstructured', 'shared-state': 'shared',
    nondeterminism: 'compound-nd', 'runtime-errors': 'compound-rt', 'data-corruption': 'compound-dc' };
  for (const [category, variable] of Object.entries(variables)) {
    assert.equal(css.match(new RegExp(`--${variable}:\\s*(#[a-fA-F0-9]{6})`))[1], PALETTE[category]);
  }
});
