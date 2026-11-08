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
