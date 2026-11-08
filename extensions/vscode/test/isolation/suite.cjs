const assert = require('node:assert/strict');
const path = require('node:path');
const vscode = require('vscode');
const registry = require('../../languages.json');
const TIMEOUT_MS = 15000;
const POLL_MS = 50;
const ACTIVATION_SETTLE_MS = 500;
async function until(predicate, detail = () => '') {
  const deadline = Date.now() + TIMEOUT_MS;
  while (!await predicate()) {
    assert.ok(Date.now() < deadline, `Timed out waiting for extension state: ${detail()}`);
    await new Promise(resolve => setTimeout(resolve, POLL_MS));
  }
}
exports.run = async () => {
  const languages = JSON.parse(process.env.STATE_LENS_ISOLATION_LANGUAGES);
  const workspace = process.env.STATE_LENS_ISOLATION_WORKSPACE;
  await vscode.commands.executeCommand('workbench.action.closeAllEditors');
  const instances = [];
  if (languages.length === 1) {
    const otherLanguage = Object.keys(registry).find(language => language !== languages[0]);
    const document = await vscode.workspace.openTextDocument(path.join(workspace, `sample.${registry[otherLanguage].extensions[0]}`));
    await vscode.window.showTextDocument(document);
    await new Promise(resolve => setTimeout(resolve, ACTIVATION_SETTLE_MS));
    assert.equal(vscode.extensions.getExtension(`state-lens.state-lens-${languages[0]}`).isActive, false);
  }
  for (const language of languages) {
    const extension = vscode.extensions.getExtension(`state-lens.state-lens-${language}`);
    assert.ok(extension);
    assert.deepEqual(extension.packageJSON.activationEvents, [`onLanguage:${language}`]);
    const document = await vscode.workspace.openTextDocument(path.join(workspace, `sample.${registry[language].extensions[0]}`));
    const api = await extension.activate();
    assert.equal(api.serverRunning(), false);
    const editor = await vscode.window.showTextDocument(document, { preview: false, viewColumn: vscode.ViewColumn.Beside });
    await until(() => api.serverRunning() && api.applied(document.uri.toString()));
    const snapshot = (await api.snapshot(document.uri.toString())).audit;
    const labels = new Set(snapshot.annotations.map(annotation => annotation.label));
    for (const label of ['Impure', 'Mutacious', 'Volatile', 'Unstructured', 'Async', 'Shared', 'Untyped']) {
      assert.ok(labels.has(label), `${language}: missing ${label}`);
    }
    if (['typescript', 'javascript', 'python'].includes(language)) {
      assert.ok(labels.has('Polymorphic'), `${language}: missing Polymorphic`);
      assert.ok(snapshot.findings.some(entry => entry.finding.resolution === 'unsafe-input-execution'));
    }
    assert.ok(snapshot.findings.some(entry => entry.finding.resolution === 'resource-lifecycle'));
    const applied = api.applied(document.uri.toString());
    assert.ok(applied.triggers > 0);
    assert.ok(applied.labels.length > 0);
    const keys = applied.labels.map(a => `${a.category}:${a.position.line}`);
    assert.equal(new Set(keys).size, keys.length);
    for (const annotation of applied.labels) assert.ok(annotation.findings.every(index => index < snapshot.findings.length));
    const configuration = vscode.workspace.getConfiguration(`stateLens.${language}`);
    instances.push({ language, api, document, configuration, column: editor.viewColumn });
  }
  for (const current of instances) {
    await vscode.window.showTextDocument(current.document, { viewColumn: current.column });
    await until(() => current.api.applied(current.document.uri.toString()));
    await vscode.commands.executeCommand(`stateLens.${current.language}.toggle`);
    await until(() => !current.api.applied(current.document.uri.toString()) && current.api.ui().lifecycle === 'idle');
    await until(() => vscode.workspace.getConfiguration(`stateLens.${current.language}`).get('enabled') === false);
    for (const other of instances.filter(instance => instance !== current)) {
      assert.equal(vscode.workspace.getConfiguration(`stateLens.${other.language}`).get('enabled'), true);
      assert.equal(other.api.serverRunning(), true);
    }
    await vscode.commands.executeCommand(`stateLens.${current.language}.toggle`);
    await until(() => current.api.applied(current.document.uri.toString()));
    await until(() => current.api.ui().statusVisible && instances.filter(instance => instance.api.ui().statusVisible).length === 1, () => JSON.stringify({ expected: current.language, active: vscode.window.activeTextEditor?.document.uri.toString(), instances: instances.map(instance => ({ language: instance.language, ...instance.api.ui() })) }));
    assert.equal(current.api.ui().statusVisible, true);
    assert.equal(current.api.ui().evidenceUri, current.document.uri.toString());
    await vscode.commands.executeCommand(`stateLens.${current.language}.evidence.focus`);
    await until(() => current.api.ui().evidenceVisible && instances.filter(instance => instance.api.ui().evidenceVisible).length === 1);
    assert.equal(current.api.ui().statusVisible, true);

  }
  const unsupported = await vscode.workspace.openTextDocument({ language: 'plaintext', content: 'unrelated' });
  await vscode.window.showTextDocument(unsupported, { viewColumn: vscode.ViewColumn.Beside });
  await until(() => instances.every(instance => !instance.api.ui().statusVisible && !instance.api.ui().evidenceVisible));
  assert.ok(instances.every(instance => instance.api.applied(unsupported.uri.toString()) === undefined));
  await vscode.commands.executeCommand('workbench.action.closeAllEditors');
  await until(() => instances.every(instance => instance.api.ui().lifecycle === 'idle' && !instance.api.ui().statusVisible));
  const current = instances[0];
  await vscode.window.showTextDocument(current.document);
  await until(() => current.api.ui().lifecycle === 'running' && current.api.ui().statusVisible);
  assert.ok(instances.slice(1).every(instance => !instance.api.serverRunning()));
  await vscode.commands.executeCommand(`stateLens.${current.language}.toggle`);
  await until(() => current.api.ui().lifecycle === 'idle');
  await vscode.workspace.getConfiguration(`stateLens.${current.language}`).update('serverPath', path.join(workspace, 'missing-state-lens'), vscode.ConfigurationTarget.Global);
  await vscode.commands.executeCommand(`stateLens.${current.language}.toggle`);
  await until(() => current.api.ui().lifecycle === 'failed');
  assert.equal(current.api.serverRunning(), false);
  await vscode.workspace.getConfiguration(`stateLens.${current.language}`).update('serverPath', undefined, vscode.ConfigurationTarget.Global);
  await vscode.commands.executeCommand(`stateLens.${current.language}.refresh`);
  await until(() => current.api.ui().lifecycle === 'running');
  await vscode.commands.executeCommand('workbench.action.closeAllEditors');
  await until(() => instances.every(instance => !instance.api.serverRunning()));
};
