const vscode = require('vscode');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const LANGUAGE = process.env.STATE_LENS_LANGUAGE;
const CONFIG_SECTION = `stateLens.${LANGUAGE}`;
const EXTENSION_ID = `state-lens.state-lens-${LANGUAGE}`;
const SOURCE = 'state-lens';
const INLAY_MARKER = '◆';
const PRESENTATION_EXAMPLE = 'examples/shared-state/async-io-misuse/go/03_goroutine_leak_unbuffered_result.go';
const DEADLINE_MS = 10000;
const STARTUP_MS = 30000;
const POLL_MS = 25;
const FAILED_STATUS = /unavailable|stopped/;
const ROOT = process.env.STATE_LENS_ROOT;
const SHARD = process.env.STATE_LENS_SHARD;
const REPORT = process.env.STATE_LENS_REPORT;
const DIVISION = JSON.parse(fs.readFileSync(path.join(ROOT, 'tooling/state-lens/fixtures/division-smoke.json'), 'utf8'));
const MUTATIONS = JSON.parse(fs.readFileSync(path.join(ROOT, 'tooling/state-lens/fixtures/division-mutations.json'), 'utf8'));
const HEALTH = process.env.STATE_LENS_HEALTH_ONLY === '1';
const EXTENSIONS = new Set(['.go', '.rs', '.ts', '.py', '.js', '.mjs', '.cjs']);
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
  const parts = file.split('/');
  const [root, family] = parts;
  const [defect, language] = parts.slice(-3);
  const compound_class = family === 'compound-defects' ? parts[2] : null;
  const manifest = compound_class ? JSON.parse(fs.readFileSync(path.join(ROOT, path.dirname(path.dirname(file)), 'manifest.json'), 'utf8')) : null;
  const example = manifest?.examples.find(e => e.file === parts.slice(-2).join('/'));
  const expected_rule_ids = !HEALTH && example ? [...example.ingredients.map(i => `${language}.${i.family}.${i.defect}`), `${language}.${family}.${compound_class}.${defect}`] : [];
  return { layer: 'editor', file, family, defect, language, root, compound_class, compound_category: manifest?.category ?? null, expected_rule_ids, missing_rule_ids: [...expected_rule_ids], status: 'pass', problems: [], findings: null,
    hinted_functions: null, ms: 0, bug_lines_hit: [], bug_lines_missed: [] };
}
function finish(row) {
  if (row.missing_rule_ids.length && !row.problems.some(p => p.category === 'count-mismatch')) row.problems.push(issue('count-mismatch', `Missing identities: ${row.missing_rule_ids.join(', ')}`));
  row.status = row.problems.length ? 'fail' : !HEALTH && row.root === 'examples' && (row.findings === 0 || row.bug_lines_missed.length) ? 'gap' : 'pass';
  return row;
}
function checkpoint(rows) {
  fs.mkdirSync(path.dirname(REPORT), { recursive: true });
  fs.writeFileSync(REPORT + '.tmp', JSON.stringify({ schema_version: 2, rows: rows.map(finish) }, null, 2));
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
  const manifest = JSON.parse(fs.readFileSync(path.join(ROOT, path.dirname(path.dirname(row.file)), 'manifest.json'), 'utf8'));
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
  if (!HEALTH && row.compound_class) {
    row.missing_rule_ids = row.expected_rule_ids.filter(id => !rules[id]);
    if (row.missing_rule_ids.length || Object.keys(rules).length < 3) row.problems.push(issue('count-mismatch', `Missing identities: ${row.missing_rule_ids.join(', ')}; expected at least three diagnostics`));
    for (const { finding } of snapshot.findings.filter(e => e.finding.rule_id === row.expected_rule_ids[2])) {
      const metadata = finding.compound;
      const ingredients = metadata?.ingredients ?? [];
      if (metadata?.class !== row.compound_class || metadata?.category !== row.compound_category || ingredients.length !== 2 || row.expected_rule_ids.slice(0, 2).some(id => !ingredients.some(e => e.rule_id === id))) row.problems.push(issue('count-mismatch', 'Compound metadata or ingredient identities differ'));
      for (const evidence of ingredients) {
        if (!snapshot.findings.some(e => e.finding.rule_id === evidence.rule_id && e.finding.evidence.some(ref => JSON.stringify(ref) === JSON.stringify(evidence)))) row.problems.push(issue('count-mismatch', `Broken linked ingredient: ${evidence.rule_id}`));
      }
    }
  }
  checkRanges(snapshot, document, row);
  if (!HEALTH) coverage(row, snapshot);
  const hash = crypto.createHash('sha256').update(fs.readFileSync(document.uri.fsPath)).digest('hex');
  if (snapshot.sourceHash !== hash) row.problems.push(issue('error', 'snapshot source hash differs from file'));
  const sorted = value => JSON.stringify(Object.entries(value).sort(([a], [b]) => a.localeCompare(b)));
  if (!HEALTH && (!expected || expected.source_hash !== hash || expected.parse_errors !== snapshot.parseErrors || expected.hinted_functions !== hinted || sorted(expected.rule_ids) !== sorted(rules))) row.problems.push(issue('baseline-drift', `snapshot differs from smoke.json: ${JSON.stringify({ source_hash: hash, parse_errors: snapshot.parseErrors, rule_ids: rules, hinted_functions: hinted })}`));
  if ((result.segments > 0) !== (hinted > 0)) row.problems.push(issue('count-mismatch', 'decoration segments differ from hinted functions'));
  const diagnostics = await until(() => lensDiagnostics(uri).length === result.findings && lensDiagnostics(uri), state, 'settled diagnostics');
  if (state.expired) return;
  for (const diagnostic of diagnostics) {
    const code = typeof diagnostic.code === 'object' ? diagnostic.code.value : diagnostic.code;
    const entry = snapshot.findings.find(e => e.finding.rule_id === code && diagnostic.range.isEqual(new vscode.Range(e.triggerRange.start.line, e.triggerRange.start.character, e.triggerRange.end.line, e.triggerRange.end.character)));
    const warning = ['known-zero', 'possible-zero'].includes(entry?.finding.division_risk?.confidence);
    if (diagnostic.severity !== (warning ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Information)) row.problems.push(issue('count-mismatch', 'Unexpected diagnostic severity'));
  }
  const expectedFindings = expected && Object.values(expected.rule_ids).reduce((a, b) => a + b, 0);
  if (!HEALTH && expected && diagnostics.length !== expectedFindings) row.problems.push(issue('count-mismatch', `${diagnostics.length} diagnostics, baseline ${expectedFindings}`));
  await until(() => api.applied(uri.toString())?.version === document.version, state, 'presentation snapshot');
  if (!HEALTH && DIVISION[row.file]) await checkDivision(api, document, DIVISION[row.file], state);
  const whole = new vscode.Range(0, 0, document.lineCount, 0);
  const hints = (await bounded(vscode.commands.executeCommand('vscode.executeInlayHintProvider', uri, whole), state, 'inlay hints') || []).filter(h => label(h).startsWith(INLAY_MARKER));
  if (state.expired) return;
  if (hints.some(hint => !/^◆ (Impure|Signals)$/.test(label(hint)))) row.problems.push(issue('error', 'verbose function summary'));
  if (hints.length !== hinted) row.problems.push(issue('count-mismatch', `${hints.length} inlay hints for ${hinted} functions`));
  const target = diagnostics.find(d => !d.range.isEmpty);
  if (target) {
    await until(() => api.applied(uri.toString())?.version === document.version, state, 'hover snapshot');
    const hovers = await bounded(vscode.commands.executeCommand('vscode.executeHoverProvider', uri, target.range.start), state, 'hover');
    if (state.expired) return;
    const guide = target.code?.target?.toString();
    const text = hoverText(hovers || []);
    if (/Susceptibility is not|Related taxonomy|\[bytes|• local_mutation •/.test(text)) row.problems.push(issue('error', 'hover contains removed metadata'));
    if (!text.includes(`command:${CONFIG_SECTION}.showEvidence`)) row.problems.push(issue('error', 'hover lacks Evidence action'));
    if (!guide || !text.includes(guide)) row.problems.push(issue('error', `hover lacks guide URL ${guide}`));
    if (!HEALTH && row.file === PRESENTATION_EXAMPLE) {
      const link = text.match(new RegExp(`command:stateLens\\.${LANGUAGE}\\.showEvidence\\?([^)]*)`));
      if (link) {
        const args = JSON.parse(decodeURIComponent(link[1]));
        await bounded(vscode.commands.executeCommand(`${CONFIG_SECTION}.showEvidence`, ...args), state, 'reveal Evidence');
        const editor = vscode.window.activeTextEditor;
        if (editor?.document !== document || !editor.selection.start.isEqual(target.range.start)) row.problems.push(issue('error', 'Evidence did not select its source'));
        const before = editor?.selection;
        await bounded(vscode.commands.executeCommand(`${CONFIG_SECTION}.showEvidence`, args[0], args[1] - 1, args[2]), state, 'reject stale Evidence');
        if (!before?.isEqual(vscode.window.activeTextEditor?.selection)) row.problems.push(issue('error', 'stale Evidence changed the selection'));
        const configuration = vscode.workspace.getConfiguration(CONFIG_SECTION);
        const originalDirectOnly = configuration.inspect('directOnly').workspaceValue;
        const previous = api.applied(uri.toString());
        try {
          await bounded(configuration.update('directOnly', true, vscode.ConfigurationTarget.Workspace), state, 'hide propagated effects');
          const filtered = await until(() => {
            const value = api.applied(uri.toString());
            return value && value !== previous && value.version === document.version && value;
          }, state, 'direct-only decorations');
          const expected = await bounded(api.snapshot(uri.toString()), state, 'direct-only snapshot');
          if (JSON.stringify(filtered.functionMarkers) !== JSON.stringify(expected?.functionMarkers) || filtered.triggers !== expected?.triggers) row.problems.push(issue('error', 'directOnly presentation differs from current findings'));
        } finally {
          const filtered = api.applied(uri.toString());
          await configuration.update('directOnly', originalDirectOnly, vscode.ConfigurationTarget.Workspace);
          await until(() => {
            const value = api.applied(uri.toString());
            return value && value !== filtered && value.version === document.version;
          }, state, 'restore all effects');
        }
      }
    }
  }
}


async function checkDivision(api, document, expected, state) {
  const same = require('node:util').isDeepStrictEqual;
  await until(async () => {
    const snapshot = await api.snapshot(document.uri.toString());
    const applied = api.applied(document.uri.toString());
    if (snapshot?.audit.version !== document.version || applied?.version !== document.version) return false;
    const entries = snapshot.audit.findings.filter(e => e.finding.rule_id === expected.rule_id);
    const diagnostics = lensDiagnostics(document.uri).filter(d => (typeof d.code === 'object' ? d.code.value : d.code) === expected.rule_id);
    const ranges = expected.findings.map(f => f.range);
    const paints = applied.decorations.filter(d => d.category === 'division-risk' || d.category === 'unstructured-state');
    return same(entries.map(e => e.triggerRange), ranges) && diagnostics.length === ranges.length
      && diagnostics.every(d => d.severity === vscode.DiagnosticSeverity.Warning)
      && same(paints.map(d => ({ range: d.range, category: d.category, outline: d.outline })), expected.findings.map(f => ({ range: f.range, category: f.category, outline: false })));
  }, state, 'division findings, warnings, and fills');
}

exports.run = async () => {
  const baseline = JSON.parse(fs.readFileSync(process.env.STATE_LENS_BASELINE, 'utf8')).files;
  const corpus = files(path.join(ROOT, SHARD)).filter(file => (!process.env.STATE_LENS_FILE || file === process.env.STATE_LENS_FILE) && (!process.env.STATE_LENS_LANGUAGE || file.split('/').at(-2) === process.env.STATE_LENS_LANGUAGE));
  if (!corpus.length) throw new Error(`No matching files in ${SHARD}`);
  const rows = corpus.map(makeRow);
  const removed = Object.keys(baseline).filter(file => !HEALTH && !process.env.STATE_LENS_FILE && !process.env.STATE_LENS_LANGUAGE && file.startsWith(SHARD + '/') && !corpus.includes(file)).map(file => {
    const row = makeRow(file);
    row.problems.push(issue('baseline-drift', 'baseline file missing from corpus'));
    return finish(row);
  });
  const groups = new Map();
  for (const row of rows) {
    const group = HEALTH ? row.file : path.posix.dirname(row.file);
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
        let editor = await bounded(vscode.window.showTextDocument(documents[0], { preview: false }), state, 'select visible editor');
        await until(() => api.serverRunning(), state, 'visible editor server startup');
        for (let index = 0; index < group.length; index++) {
          const row = group[index];
          await bounded(vscode.window.showTextDocument(documents[index], { preview: false }), state, 'activate document');
          try { await check(api, documents[index], row, baseline[row.file], state); }
          catch (error) { if (!state.expired || error.category === 'timeout') row.problems.push(issue(error.category || 'error', error.message)); }
        }
        if (HEALTH) {
          const cleanup = { deadline: Date.now() + DEADLINE_MS, expired: false };
          await bounded(vscode.commands.executeCommand('workbench.action.closeAllEditors'), cleanup, 'close document');
          continue;
        }
        for (let index = 0; index < group.length; index++) {
          const row = group[index];
          const document = documents[index];
          await bounded(vscode.window.showTextDocument(document, { preview: false }), state, 'activate presentation');
          const actual = await until(() => {
            const value = api.applied(document.uri.toString());
            return value?.version === document.version && value;
          }, state, 'source decorations');
          const audit = (await bounded(api.snapshot(document.uri.toString()), state, 'marker snapshot'))?.audit;
          const effects = new Set(['local_mutation', 'argument_mutation', 'receiver_mutation', 'global_mutation', 'external_read', 'nondeterminism']);
          const expectedMarkers = audit?.functions.filter(fn => audit.findings.some(entry => JSON.stringify(entry.functionRange) === JSON.stringify(fn.range) && effects.has(entry.finding.effect.kind))).map(fn => ({ start: fn.range.start, end: fn.body.start }));
          if (JSON.stringify(actual.functionMarkers) !== JSON.stringify(expectedMarkers)) row.problems.push(issue('count-mismatch', 'Applied function markers differ from owned effects'));
          if (audit?.findings.length && !actual.decorations.length) row.problems.push(issue('count-mismatch', 'Missing source decorations'));
          if (!row.compound_class) continue;
          const required = [...row.expected_rule_ids.slice(0, 2).map(id => id.split('.')[1]), row.compound_class];
          for (const category of required) {
            if (!actual.decorations.some(d => d.category === category || d.families.includes(category))) row.problems.push(issue('count-mismatch', `Missing painted category: ${category}`));
          }
          if (!actual.decorations.some(d => d.category === row.compound_class && d.outline)) row.problems.push(issue('count-mismatch', 'Missing compound outline'));
        }
        for (let index = 0; index < group.length; index++) {
          const row = group[index];
          if (!DIVISION[row.file] || row.root !== 'examples') continue;
          const target = await bounded(vscode.window.showTextDocument(documents[index], { preview: false }), state, 'select division example');
          edited = target.document;
          original = edited.getText();
          const fixedFile = row.file.replace('examples/', 'examples-fixed/');
          await target.edit(edit => edit.replace(new vscode.Range(0, 0, edited.lineCount, 0), fs.readFileSync(path.join(ROOT, fixedFile), 'utf8')));
          await checkDivision(api, edited, DIVISION[fixedFile], state);
          await target.edit(edit => edit.replace(new vscode.Range(0, 0, edited.lineCount, 0), original));
          await checkDivision(api, edited, DIVISION[row.file], state);
          const mutation = MUTATIONS[row.language];
          await target.edit(edit => edit.replace(new vscode.Range(0, 0, edited.lineCount, 0), mutation.source));
          await checkDivision(api, edited, mutation, state);
          await target.edit(edit => edit.replace(new vscode.Range(0, 0, edited.lineCount, 0), original));
          await checkDivision(api, edited, DIVISION[row.file], state);
          edited = undefined;
        }
        editor = await bounded(vscode.window.showTextDocument(documents[0], { preview: false }), state, 'restore active editor');
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
        const errors = logProblems(api.log(logCursor));
        logCursor = api.logCount ? api.logCount() : logs.length;
        for (const row of group) {
          row.problems.push(...errors);
          if (api.ui().lifecycle === 'failed') row.problems.push(issue('crash', 'language server failed'));
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
      if (aborted || api.ui().lifecycle === 'failed') break;
    }
  } catch (error) {
    for (const row of rows.filter(row => !processed.has(row.file))) row.problems.push(issue(error.category || 'crash', error.message));
  } finally {
    for (const row of rows) {
      if (!processed.has(row.file) && !row.problems.length) row.problems.push(issue('crash', 'not executed after shard failure'));
    }
    if (api) {
      fs.writeFileSync(path.join(path.dirname(REPORT), 'state-lens.log'), api.log().join('\n'));
      const tail = logProblems(api.log(logCursor));
      for (const row of rows) row.problems.push(...tail);
    }
    checkpoint([...rows, ...removed]);
  }
  const failures = [...rows, ...removed].filter(row => row.status === 'fail');
  console.log(`${SHARD}: ${rows.length} files, ${failures.length} failures`);
  if (failures.length || process.env.STATE_LENS_STRICT_GAPS === '1' && rows.some(row => row.status === 'gap')) throw new Error('State Lens smoke failed; see audit report');
};
