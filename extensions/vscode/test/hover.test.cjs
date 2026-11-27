const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const vm = require('node:vm');

class MarkdownString {
  parts = [];
  appendText(value) { this.parts.push({ kind: 'text', value }); return this; }
  appendMarkdown(value) { this.parts.push({ kind: 'markdown', value }); return this; }
}
const moduleStub = { exports: {} };
vm.runInNewContext(readFileSync(join(__dirname, '../dist/hover.js'), 'utf8'), {
  module: moduleStub,
  require(name) {
    if (name === './package') return require('../dist/package');
    if (name === './ranges') return require('../dist/ranges');
    assert.equal(name, 'vscode');
    return { MarkdownString, Uri: { parse: uri => ({ fsPath: new URL(uri).pathname }) } };
  }
});
const { explanation } = moduleStub.exports;

function entry(family, defect, message = 'Modeled mutation of an explicitly typed receiver.') {
  return { finding: {
    family, defect, explanation: message,
    origin: 'direct', effect: { kind: 'argument_mutation' }, resolution: 'modeled-api',
    related_cwes: ['CWE-374'], call_path: [],
    evidence: [{ uri: 'file:///examples/mutated-state/for-loop-control-flow/example.ts', range: { start: 169, end: 183 } }]
  } };
}

test('hover guide links follow findings rather than the evidence directory', () => {
  for (const [family, defect] of [
    ['mutated-state', 'impure-functions'],
    ['mutated-state', 'var-let-mut'],
    ['unstructured-state', 'type-errors'],
    ['shared-state', 'threading-mutexes']
  ]) {
    const hover = explanation(entry(family, defect));
    const links = hover.parts.filter(p => p.kind === 'markdown');
    assert.equal(links.length, 2);
    assert.equal(links[1].value, `\n\n[Guide](<https://badsoftware.untra.io/en/${family}/${defect}/>)`);
    const content = hover.parts.map(part => part.value).join('');
    assert.doesNotMatch(content, /Susceptibility|Related taxonomy|bytes|file:\/\//);
    assert.doesNotMatch(content, /direct|argument_mutation|modeled-api/);
  }
});

test('explanatory content remains literal and the guide needs no trusted commands', () => {
  const message = '[Open](command:unexpected) <b>text</b> *emphasis*';
  const hover = explanation(entry('mutated-state', 'impure-functions', message));
  assert.ok(hover.parts.some(p => p.kind === 'text' && p.value.includes(message)));
  assert.ok(hover.parts.filter(p => p.kind === 'markdown').every(p => !p.value.includes(message)));
  assert.equal(hover.isTrusted, undefined);
});

test('compound hover names its category and uses the class route', () => {
  const value = entry('compound-defects', 'lost-update', 'An increment writes back a shared snapshot.');
  value.finding.compound = { class: 'nondeterminism', category: 'temporal-interleaving', ingredients: [] };
  const hover = explanation(value);
  assert.ok(hover.parts.some(p => p.value.includes('lost-update')));
  assert.ok(hover.parts.some(p => p.value.includes('#C9A227')));
  assert.ok(hover.parts.some(p => p.value.includes('/en/compound-defects/nondeterminism/lost-update/')));
});

test('family titles use the palette and escape HTML', () => {
  const { PALETTE } = require('../dist/ranges');
  for (const family of ['mutated-state', 'unstructured-state', 'shared-state']) {
    const hover = explanation(entry(family, '<img src=x>'));
    assert.equal(hover.supportHtml, true);
    assert.ok(hover.parts[0].value.includes(PALETTE[family]));
    assert.ok(hover.parts[0].value.includes('&lt;img src=x&gt;'));
  }
});
test('Evidence links carry the snapshot identity and trust only the Evidence command', () => {
  const value = entry('mutated-state', 'var-let-mut', 'Mutable binding; local mutation does not imply observable impurity.');
  const hover = explanation(value, 'file:///example.ts', 7);
  const link = hover.parts.at(-1).value;
  assert.match(link, /command:stateLens.typescript.showEvidence/);
  assert.deepEqual(JSON.parse(decodeURIComponent(link.split('?')[1].slice(0, -1))), ['file:///example.ts', 7, value.finding]);
  assert.equal(JSON.stringify(hover.isTrusted), JSON.stringify({ enabledCommands: ['stateLens.typescript.showEvidence'] }));
  assert.ok(hover.parts.some(part => part.value === '\n\nMutable binding.'));
});

test('object-identity hovers distinguish candidate relationships from declared requirements', () => {
  const trace = { evidence_level: 'feasible_under_model', origins: [{}], invalidations: [{}], affected_uses: [{}], assumptions: [], required_condition: 'independent' };
  const make = requirement => ({ finding: {
    family: 'mutated-state', defect: 'aliasing-mutable-defaults', explanation: 'Write through a shallow copy.', origin: 'direct',
    effect: { kind: 'local_mutation' }, resolution: 'object-identity', mechanism: 'shallow-copy-nested', call_path: [], evidence: [],
    state_trace: { ...trace, requirement }
  } });
  const text = hover => hover.parts.map(p => p.value).join('');
  assert.match(text(explanation(make(undefined))), /Mechanism: shallow-copy-nested\. No declared isolation requirement/);
  assert.match(text(explanation(make({ kind: 'declared-readonly', condition: 'c', declarations: [] }))), /Declared requirement \(declared-readonly\) is violated/);
});
test('shared-state hovers distinguish static sites from runtime fan-out', () => {
  const value = entry('shared-state', 'race-conditions');
  value.finding.shared_state = { evidence_level: 'relationship-candidate', sites: { declarations: 1, launches: 2, suspensions: 3, dynamic_fan_out: true }, contenders: 2, events: [{kind:'read'}, {kind:'suspend'}, {kind:'write'}], assumptions: ['Unknown aliases remain unresolved.'] };
  const text = explanation(value).parts.map(p => p.value).join('');
  assert.match(text, /Static sites: 1 async declarations, 2 launches\/compositions, 3 suspensions/);
  assert.match(text, /Runtime fan-out is unknown/);
  assert.match(text, /read → suspend → write/);
});

test('mutex hovers distinguish usage, candidates and misuse without new labels', () => {
  for (const assessment of ['usage', 'candidate', 'misuse']) {
    const value = entry('shared-state', 'threading-mutexes', 'Recognized critical section.');
    value.finding.origin = 'propagated';
    value.finding.shared_state = {
      evidence_level: 'relationship-candidate', sites: { declarations: 0, launches: 0, suspensions: 0, dynamic_fan_out: false },
      contenders: 0, events: [], assumptions: [],
      synchronization: { api: 'threading.Lock', lock: 'ledger', assessment, acquisitions: [{ start: 3, end: 8 }], releases: [], protected_regions: [] }
    };
    const content = explanation(value).parts.map(part => part.value).join('');
    assert.ok(content.includes(`Synchronization: ${assessment}`));
    assert.ok(content.includes('does not mean the caller holds its lock'));
  }
});

test('effect assessment and potential overlaps remain distinct from diagnoses', () => {
  const usage = entry('shared-state', 'cache-invalidation');
  usage.finding.assessment = 'usage';
  assert.ok(explanation(usage).parts.some(part => part.value.includes('Assessment: usage.')));
  const overlap = entry('compound-defects', 'nondeterminism', 'Potential overlap only.');
  overlap.finding.assessment = 'candidate';
  overlap.finding.compound = { class: 'nondeterminism', category: 'potential-overlap', ingredients: [] };
  const hover = explanation(overlap);
  assert.ok(hover.parts.some(part => part.value.includes('Assessment: candidate.')));
  assert.ok(hover.parts.some(part => part.value.includes('/en/compound-defects/nondeterminism/>')));
});
