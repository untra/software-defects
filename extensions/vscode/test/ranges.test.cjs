const { test } = require('node:test');
const assert = require('node:assert/strict');
const { segments, accepts } = require('../dist/ranges');
const r = (start, end) => ({ start: { line: start, character: 0 }, end: { line: end, character: 0 } });
const finding = (fn, origin = 'direct') => ({ functionRange: fn, triggerRange: r(1, 2), blastRadius: [r(1, 2)], finding: { origin } });
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
  const entry = (family, start, end, compound) => ({ triggerRange: r(start, end), blastRadius: [r(start, end)], functionRange: null,
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
    triggerRange: r(1, 2), blastRadius: [r(1, 2)], functionRange: null, finding: { family, rule_id: family, origin: 'direct' }
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
    nondeterminism: 'compound', 'runtime-errors': 'compound', 'data-corruption': 'compound' };
  for (const [category, variable] of Object.entries(variables)) {
    assert.equal(css.match(new RegExp(`--${variable}:\\s*(#[a-fA-F0-9]{6})`))[1], PALETTE[category]);
  }
});

test('function markers preserve server categories and direct-only evidence', () => {
  const { functionMarkers } = require('../dist/ranges');
  const outer = r(0, 10);
  const snapshot = { functions: [{range: outer, body: r(1,10)}], findings: [finding(outer), finding(outer, 'propagated')], annotations: [
    {position: {line:0,character:0},target:'function',category:'mutated-state',label:'Impure',range:r(0,1),findings:[0]},
    {position: {line:0,character:0},target:'function',category:'shared-state',label:'Shared',range:r(0,1),findings:[1]}
  ] };
  assert.deepEqual(functionMarkers(snapshot,false).map(m => m.label), ['Impure','Shared']);
  assert.deepEqual(functionMarkers(snapshot,true).map(m => m.category), ['mutated-state']);
  assert.deepEqual(functionMarkers({...snapshot,annotations:undefined},false), []);
});

test('base division retains its family color; gold requires a compound recipe', () => {
  const { decorations } = require('../dist/ranges');
  const entry = { triggerRange: r(1, 2), blastRadius: [r(1, 2)], functionRange: null, finding: {
    rule_id: 'typescript.unstructured-state.divide-by-zero', family: 'unstructured-state', origin: 'direct',
    division_risk: { confidence: 'known-zero', outcome: 'non-finite', mutated: false }
  } };
  const snapshot = { functions: [], findings: [entry] };
  assert.equal(decorations(snapshot, false)[0].category, 'unstructured-state');
  entry.finding.division_risk.mutated = true;
  assert.equal(decorations(snapshot, false)[0].category, 'unstructured-state');
  entry.finding.division_risk.confidence = 'candidate';
  assert.equal(decorations(snapshot, false)[0].category, 'unstructured-state');
});

test('division categories use fills and stale servers fail the capability check', () => {
  const { CATEGORY_STYLE, divisionCompatible } = require('../dist/ranges');
  assert.equal(CATEGORY_STYLE['division-risk'], 'fill');
  assert.equal(CATEGORY_STYLE['unstructured-state'], 'fill');
  assert.equal(CATEGORY_STYLE['runtime-errors'], 'outline');
  assert.equal(divisionCompatible(undefined), false);
  assert.equal(divisionCompatible({}), false);
  assert.equal(divisionCompatible({ experimental: { stateLens: { divisionAnalysis: 1 } } }), true);
});

test('annotation filtering preserves snapshot indices and hides propagated tooltip evidence', () => {
  const { annotations } = require('../dist/ranges');
  const direct = finding(r(0, 2));
  const propagated = finding(r(0, 2), 'propagated');
  const snapshot = { findings: [propagated, direct], annotations: [
    { position: {line:0,character:0}, range:r(0,1), category: 'mutated-state', target: 'function', label: 'Impure', findings: [0, 1] },
    { position: {line:0,character:0}, range:r(0,1), category: 'shared-state', target: 'property', label: 'Shared', findings: [0] }
  ] };
  assert.deepEqual(annotations(snapshot, true).map(a => a.findings), [[1]]);
  assert.equal(snapshot.findings[1], direct);
  assert.deepEqual(snapshot.annotations[0].findings, [0, 1]);
  assert.deepEqual(annotations(snapshot, false).map(a => a.findings), [[0, 1], [0]]);
});

test('same family merges after visibility filtering at the narrowest source', () => {
  const { annotations } = require('../dist/ranges');
  const snapshot = { findings: [finding(r(0,2)), finding(r(0,2), 'propagated')], annotations: [
    {category:'mutated-state',target:'function',label:'Impure',range:r(0,2),position:{line:0,character:0},findings:[0]},
    {category:'mutated-state',target:'property',label:'Volatile',range:{start:{line:0,character:8},end:{line:0,character:9}},position:{line:0,character:9},findings:[1]}
  ]};
  assert.equal(annotations(snapshot,false).length,1);
  assert.equal(annotations(snapshot,false)[0].label,'Mutacious');
  assert.equal(annotations(snapshot,false)[0].position.character,9);
  assert.equal(annotations(snapshot,true)[0].label,'Impure');
  assert.equal(annotations(snapshot,true)[0].position.character,0);
});

test('unsupported blast radius does not invent a function highlight', () => {
  const { decorations } = require('../dist/ranges');
  assert.deepEqual(decorations({functions:[],findings:[{...finding(r(0,10)),blastRadius:[],finding:{family:'mutated-state',origin:'direct'}}]},false),[]);
});

test('trace evidence retains distinct roles and source hashes without inventing omissions', () => {
  const { findingEvidence } = require('../dist/ranges');
  const origin = { uri: 'file:///a.ts', range: { start: 0, end: 1 } };
  const use = { uri: 'file:///a.ts', range: { start: 20, end: 21 } };
  const evidence = findingEvidence({ rule_id: 'rule', call_path: [], evidence: [], state_trace: {
    source_hash: 'snapshot', origins: [origin], invalidations: [], affected_uses: [use]
  } });
  assert.deepEqual(evidence.map(e => e.role), ['Origin', 'Affected use']);
  assert.ok(evidence.every(e => e.source_hash === 'snapshot'));
  assert.deepEqual(evidence.map(e => e.range), [origin.range, use.range]);
});
