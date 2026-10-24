const vscode = require('vscode');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const EXTENSION_ID = 'state-lens.state-lens';
const SOURCE = 'state-lens';
const INLAY_MARKER = '◆';
const DEADLINE_MS = 10000;
const STARTUP_MS = 30000;
const POLL_MS = 25;
const FAILED_STATUS = /unavailable|stopped/;
const ROOT = process.env.STATE_LENS_ROOT;
const SHARD = process.env.STATE_LENS_SHARD;
const REPORT = process.env.STATE_LENS_REPORT;
const EXTENSIONS = new Set(['.go', '.rs', '.ts']);
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
const issue = (category, message) => ({ category, message: String(message) });
const label = hint => typeof hint.label === 'string' ? hint.label : hint.label.map(part => part.value).join('');
const hoverText = hovers => hovers.flatMap(h => h.contents).map(c => typeof c === 'string' ? c : c.value).join('\n');
const lensDiagnostics = uri => vscode.languages.getDiagnostics(uri).filter(d => d.source === SOURCE);

function logProblems(lines) {
  return lines.flatMap((line, index) => {
    if (!/\[(Error|Warn)/i.test(line)) return [];
    const next = lines.slice(index + 1).findIndex(value => /^\[/.test(value));
    const end = next < 0 ? lines.length : index + 1 + next;
    return [issue(/\[Error/i.test(line) ? 'log-error' : 'log-warning', lines.slice(index, end).join('\n'))];
  });
}
function files(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const file = path.join(directory, entry.name);
    return entry.isDirectory() ? files(file) : EXTENSIONS.has(path.extname(file)) ? [path.relative(ROOT, file).split(path.sep).join('/')] : [];
  }).sort();
}
function makeRow(file) {
  const [root, family, defect, language] = file.split('/');
  return { layer: 'editor', file, family, defect, language, root, status: 'pass', problems: [], findings: null,
    hinted_functions: null, ms: 0, bug_lines_hit: [], bug_lines_missed: [] };
}
function finish(row) {
  row.status = row.problems.length ? 'fail' : row.root === 'examples' && (row.hinted_functions === 0 || row.bug_lines_missed.length) ? 'gap' : 'pass';
  return row;
}
function checkpoint(rows) {
  fs.mkdirSync(path.dirname(REPORT), { recursive: true });
  fs.writeFileSync(REPORT + '.tmp', JSON.stringify({ schema_version: 1, rows: rows.map(finish) }, null, 2));
  fs.renameSync(REPORT + '.tmp', REPORT);
}
function bounded(promise, state, what) {
  const remaining = state.deadline - Date.now();
  if (state.expired || remaining <= 0) return Promise.reject(Object.assign(new Error(`timeout: ${what}`), { category: 'timeout' }));
  let timer;
  return Promise.race([promise, new Promise((_, reject) => {
    timer = setTimeout(() => { state.expired = true; reject(Object.assign(new Error(`timeout: ${what}`), { category: 'timeout' })); }, remaining);
  })]).finally(() => clearTimeout(timer));
}
async function until(probe, state, what) {
  while (!state.expired && Date.now() < state.deadline) {
    const value = await bounded(Promise.resolve().then(probe), state, what);
    if (value) return value;
    await sleep(POLL_MS);
  }
  throw Object.assign(new Error(`timeout: ${what}`), { category: 'timeout' });
}
function checkRanges(snapshot, document, row) {
  const key = p => document.offsetAt(new vscode.Position(p.line, p.character));
  const valid = p => Number.isInteger(p.line) && Number.isInteger(p.character) && p.line >= 0 && p.line < document.lineCount && p.character >= 0 && p.character <= document.lineAt(p.line).text.length;
  const ranges = snapshot.functions.flatMap(f => [f.range, f.body]).concat(snapshot.findings.flatMap(e => [e.triggerRange, ...(e.functionRange ? [e.functionRange] : [])]));
  for (const range of ranges) {
    if (!valid(range.start) || !valid(range.end) || key(range.start) > key(range.end)) row.problems.push(issue('range', `Invalid range ${JSON.stringify(range)}`));
  }
  for (const entry of snapshot.findings) {
    if (entry.functionRange && (key(entry.functionRange.start) > key(entry.triggerRange.start) || key(entry.triggerRange.end) > key(entry.functionRange.end))) row.problems.push(issue('range', 'trigger outside its function'));
  }
}
function coverage(row, snapshot) {
  if (row.root !== 'examples') return;
  const manifest = JSON.parse(fs.readFileSync(path.join(ROOT, row.root, row.family, row.defect, 'manifest.json'), 'utf8'));
  const lines = manifest.examples.find(e => e.file === row.file.split('/').slice(-2).join('/')).bug_lines;
  const hit = line => snapshot.findings.some(({ triggerRange: r }) => {
    if (r.start.line === r.end.line && r.start.character === r.end.character) return false;
    return line >= r.start.line + 1 && line <= r.end.line + Number(r.end.character > 0);
  });
  row.bug_lines_hit = [...new Set(lines.filter(hit))].sort((a, b) => a - b);
  row.bug_lines_missed = [...new Set(lines.filter(line => !hit(line)))].sort((a, b) => a - b);
}
async function check(api, document, row, expected, state) {
  const uri = document.uri;
  const result = await until(() => api.snapshot(uri.toString()), state, 'snapshot');
  if (state.expired) return;
  const snapshot = result.audit;
  const rules = {};
  for (const entry of snapshot.findings) rules[entry.finding.rule_id] = (rules[entry.finding.rule_id] || 0) + 1;
  const hinted = new Set(snapshot.findings.filter(e => e.functionRange).map(e => JSON.stringify(e.functionRange))).size;
  row.findings = result.findings;
  row.hinted_functions = hinted;
  checkRanges(snapshot, document, row);
  coverage(row, snapshot);
  const hash = crypto.createHash('sha256').update(fs.readFileSync(document.uri.fsPath)).digest('hex');
  if (snapshot.sourceHash !== hash) row.problems.push(issue('error', 'snapshot source hash differs from file'));
  const sorted = value => JSON.stringify(Object.entries(value).sort(([a], [b]) => a.localeCompare(b)));
  if (!expected || expected.source_hash !== hash || expected.parse_errors !== snapshot.parseErrors || expected.hinted_functions !== hinted || sorted(expected.rule_ids) !== sorted(rules)) row.problems.push(issue('baseline-drift', `snapshot differs from smoke.json: ${JSON.stringify({ source_hash: hash, parse_errors: snapshot.parseErrors, rule_ids: rules, hinted_functions: hinted })}`));
  if ((result.segments > 0) !== (hinted > 0)) row.problems.push(issue('count-mismatch', 'decoration segments differ from hinted functions'));
  const diagnostics = await until(() => lensDiagnostics(uri).length === result.findings && lensDiagnostics(uri), state, 'settled diagnostics');
  if (state.expired) return;
  const expectedFindings = expected && Object.values(expected.rule_ids).reduce((a, b) => a + b, 0);
  if (expected && diagnostics.length !== expectedFindings) row.problems.push(issue('count-mismatch', `${diagnostics.length} diagnostics, baseline ${expectedFindings}`));
  const whole = new vscode.Range(0, 0, document.lineCount, 0);
  const hints = (await bounded(vscode.commands.executeCommand('vscode.executeInlayHintProvider', uri, whole), state, 'inlay hints') || []).filter(h => label(h).startsWith(INLAY_MARKER));
  if (state.expired) return;
  if (hints.length !== hinted) row.problems.push(issue('count-mismatch', `${hints.length} inlay hints for ${hinted} functions`));
  const target = diagnostics.find(d => !d.range.isEmpty);
  if (target) {
    const hovers = await bounded(vscode.commands.executeCommand('vscode.executeHoverProvider', uri, target.range.start), state, 'hover');
    if (state.expired) return;
    const guide = target.code?.target?.toString();
    if (!guide || !hoverText(hovers || []).includes(guide)) row.problems.push(issue('error', `hover lacks guide URL ${guide}`));
  }
}

exports.run = async () => {
  const baseline = JSON.parse(fs.readFileSync(process.env.STATE_LENS_BASELINE, 'utf8')).files;
  const corpus = files(path.join(ROOT, SHARD));
  const rows = corpus.map(makeRow);
  const removed = Object.keys(baseline).filter(file => file.startsWith(SHARD + '/') && !corpus.includes(file)).map(file => {
    const row = makeRow(file);
    row.problems.push(issue('baseline-drift', 'baseline file missing from corpus'));
    return finish(row);
  });
  const groups = new Map();
  for (const row of rows) {
    const group = path.posix.dirname(row.file);
    if (!groups.has(group)) groups.set(group, []);
    groups.get(group).push(row);
  }
  let api;
  let aborted = false;
  let logCursor = 0;
  const processed = new Set();
  try {
    const startup = { deadline: Date.now() + STARTUP_MS, expired: false };
    api = await bounded(vscode.extensions.getExtension(EXTENSION_ID).activate(), startup, 'extension activation');
    await until(() => api.serverRunning(), startup, 'server startup');
    for (const group of groups.values()) {
      const started = Date.now();
      const state = { deadline: started + DEADLINE_MS, expired: false };
      let edited;
      let original;
      try {
        const documents = [];
        for (const row of group) {
          const document = await bounded(vscode.workspace.openTextDocument(vscode.Uri.file(path.join(ROOT, row.file))), state, 'open document');
          await bounded(vscode.window.showTextDocument(document, { preview: false, preserveFocus: true }), state, 'open tab');
          documents.push(document);
        }
        const editor = await bounded(vscode.window.showTextDocument(documents[0], { preview: false }), state, 'select visible editor');
        await Promise.all(group.map((row, index) => check(api, documents[index], row, baseline[row.file], state).catch(error => {
          if (!state.expired || error.category === 'timeout') row.problems.push(issue(error.category || 'error', error.message));
        })));
        const painted = () => api.applied(editor.document.uri.toString())?.version === editor.document.version;
        await until(painted, state, 'visible decorations');
        edited = editor.document;
        original = edited.getText();
        if (!await bounded(editor.edit(edit => edit.insert(new vscode.Position(0, 0), '\n')), state, 'edit')) throw new Error('edit rejected');
        await until(() => api.snapshot(edited.uri.toString()), state, 'edited snapshot');
        await until(painted, state, 'edited decorations');
        if (!await bounded(editor.edit(edit => edit.delete(new vscode.Range(0, 0, 1, 0))), state, 'restore')) throw new Error('restore rejected');
        await until(() => api.snapshot(edited.uri.toString()), state, 'restored snapshot');
        await until(painted, state, 'restored decorations');
        if (edited.getText() !== original) throw new Error('original text was not restored');
        await bounded(vscode.commands.executeCommand('workbench.action.revertAndCloseActiveEditor'), state, 'close restored document');
        edited = undefined;
        await bounded(vscode.commands.executeCommand('workbench.action.closeAllEditors'), state, 'close group');
      } catch (error) {
        for (const row of group) row.problems.push(issue(error.category || 'error', error.message));
        aborted = true;
      } finally {
        state.expired = true;
        const logs = api.log();
        const errors = logProblems(logs.slice(logCursor));
        logCursor = logs.length;
        for (const row of group) {
          row.problems.push(...errors);
          if (!api.serverRunning()) row.problems.push(issue('crash', 'language server stopped'));
          if (FAILED_STATUS.test(api.status())) row.problems.push(issue('error', api.status()));
          row.ms = Date.now() - started;
          processed.add(row.file);
        }
        checkpoint(rows.filter(row => processed.has(row.file)));
        if (edited) {
          const cleanup = { deadline: Date.now() + DEADLINE_MS, expired: false };
          try { await bounded(vscode.commands.executeCommand('workbench.action.revertAndCloseActiveEditor'), cleanup, 'cleanup edit'); }
          catch { aborted = true; }
        }
      }
      if (aborted || !api.serverRunning()) break;
    }
  } catch (error) {
    for (const row of rows.filter(row => !processed.has(row.file))) row.problems.push(issue(error.category || 'crash', error.message));
  } finally {
    for (const row of rows) {
      if (!processed.has(row.file) && !row.problems.length) row.problems.push(issue('crash', 'not executed after shard failure'));
    }
    if (api) {
      const tail = logProblems(api.log().slice(logCursor));
      for (const row of rows) row.problems.push(...tail);
    }
    checkpoint([...rows, ...removed]);
  }
  const failures = [...rows, ...removed].filter(row => row.status === 'fail');
  console.log(`${SHARD}: ${rows.length} files, ${failures.length} failures`);
  if (failures.length || process.env.STATE_LENS_STRICT_GAPS === '1' && rows.some(row => row.status === 'gap')) throw new Error('State Lens smoke failed; see audit report');
};
