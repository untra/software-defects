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
    assert.equal(links.length, 1);
    assert.equal(links[0].value, `\n\n[Read the defect guide ↗](<https://badsoftware.untra.io/en/${family}/${defect}/>)`);
    const linkIndex = hover.parts.indexOf(links[0]);
    assert.match(hover.parts[linkIndex - 1].value, /Susceptibility is not a bug diagnosis/);
    assert.match(hover.parts[linkIndex + 1].value, /Related taxonomy/);
  }
});

test('explanatory content remains literal and the guide needs no trusted commands', () => {
  const message = '[Open](command:unexpected) <b>text</b> *emphasis*';
  const hover = explanation(entry('mutated-state', 'impure-functions', message));
  assert.ok(hover.parts.some(p => p.kind === 'text' && p.value.includes(message)));
  assert.ok(hover.parts.filter(p => p.kind === 'markdown').every(p => !p.value.includes(message)));
  assert.equal(hover.isTrusted, undefined);
});
