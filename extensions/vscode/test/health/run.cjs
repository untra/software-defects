const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { createHash } = require('node:crypto');
const { performance } = require('node:perf_hooks');
const { Client } = require('./client.cjs');

const ROOT = path.resolve(__dirname, '../../../..');
const LANGUAGES = { '.go': 'go', '.rs': 'rust', '.ts': 'typescript', '.py': 'python' };
const DEADLINE_MS = 5000;
const STARTUP_MS = 30000;
const CONTENT_MODIFIED = -32801;
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
function files(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const file = path.join(directory, entry.name);
    return entry.isDirectory() ? files(file) : LANGUAGES[path.extname(file)] ? [file] : [];
  }).sort();
}
function executable(command) {
  if (command.includes(path.sep)) return path.resolve(command);
  for (const folder of (process.env.PATH || '').split(path.delimiter)) {
    const file = path.join(folder, command);
    try { fs.accessSync(file, fs.constants.X_OK); return fs.realpathSync(file); } catch {}
  }
  throw new Error(`Executable not found on PATH: ${command}`);
}
async function main() {
  let command = process.env.STATE_LENS_SERVER || 'state-lens';
  let report = path.resolve('health-report.json');
  let workspace = ROOT;
  let language;
  const selected = [];
  const args = process.argv.slice(2);
  while (args.length) {
    const arg = args.shift();
    const value = args.shift();
    if (!value) throw new Error(`Missing value for ${arg}`);
    if (arg === '--server') command = value;
    else if (arg === '--report') report = path.resolve(value);
    else if (arg === '--file') selected.push(path.resolve(value));
    else if (arg === '--language' && Object.values(LANGUAGES).includes(value)) language = value;
    else if (arg === '--workspace') workspace = path.resolve(value);
    else throw new Error(`Unknown argument: ${arg} ${value}`);
  }
  const corpus = (selected.length ? selected : ['examples', 'examples-fixed'].flatMap(root => files(path.join(ROOT, root))))
    .filter(file => !language || LANGUAGES[path.extname(file)] === language);
  if (!corpus.length || corpus.some(file => !LANGUAGES[path.extname(file)])) throw new Error('Select supported source files');
  const rows = corpus.map(file => ({ file: path.relative(ROOT, file), language: LANGUAGES[path.extname(file)], status: 'not-run' }));
  const output = { schema_version: 1, started: new Date().toISOString(), command, workspace, deadline_ms: DEADLINE_MS, rows };
  const checkpoint = () => {
    fs.mkdirSync(path.dirname(report), { recursive: true });
    fs.writeFileSync(report + '.tmp', JSON.stringify(output, null, 2));
    fs.renameSync(report + '.tmp', report);
  };
  let client;
  try {
    output.server = executable(command);
    output.server_sha256 = createHash('sha256').update(fs.readFileSync(output.server)).digest('hex');
    client = new Client(output.server);
    const initialize = await client.request('initialize', {
      capabilities: {}, workspaceFolders: [{ uri: pathToFileURL(workspace).href, name: path.basename(workspace) }],
      initializationOptions: { diagnostics: true }
    }, STARTUP_MS);
    output.server_info = initialize.serverInfo;
    client.notify('initialized', {});
    for (const [index, file] of corpus.entries()) {
      const row = rows[index];
      const started = performance.now();
      const uri = pathToFileURL(file).href;
      const source = fs.readFileSync(file, 'utf8');
      const logStart = client.logs.length;
      try {
        client.notify('textDocument/didOpen', { textDocument: { uri, languageId: row.language, version: 1, text: source } });
        let snapshot;
        while (!snapshot) {
          const remaining = Math.floor(DEADLINE_MS - (performance.now() - started));
          if (remaining <= 0) throw new Error('analysis deadline exceeded');
          try { snapshot = await client.request('stateLens/ranges', { textDocument: { uri } }, remaining); }
          catch (error) { if (error.code !== CONTENT_MODIFIED) throw error; await sleep(25); }
        }
        if (snapshot.uri !== uri || snapshot.version !== 1 || snapshot.sourceHash !== createHash('sha256').update(source).digest('hex')) throw new Error('stale or mismatched snapshot');
        if (!Array.isArray(snapshot.findings) || !Array.isArray(snapshot.functions)) throw new Error('invalid snapshot');
        const errors = client.logs.slice(logStart).filter(log => log.params?.type <= 2);
        if (errors.length) throw new Error(JSON.stringify(errors));
        Object.assign(row, { status: 'pass', findings: snapshot.findings.length, indexing: snapshot.indexing, parse_errors: snapshot.parseErrors, unresolved: snapshot.unresolved.length });
      } catch (error) {
        Object.assign(row, { status: 'fail', error: error.message });
      } finally {
        row.ms = Math.round(performance.now() - started);
        row.logs = client.logs.slice(logStart);
        checkpoint();
      }
      if (client.failure || row.status === 'fail' && /timeout|deadline/.test(row.error)) break;
      client.notify('textDocument/didClose', { textDocument: { uri } });
    }
    if (!client.failure) {
      await client.request('shutdown', null, DEADLINE_MS);
      client.notify('exit', null);
      await Promise.race([client.closed, sleep(DEADLINE_MS).then(() => { throw new Error('server did not exit after shutdown'); })]);
      if (client.process.exitCode !== 0) throw new Error(`server shutdown exit: ${client.process.exitCode}`);
    }
  } catch (error) { output.error = error.message; }
  finally {
    if (client) { await client.stop(); output.logs = client.logs; }
    output.finished = new Date().toISOString();
    checkpoint();
  }
  const summary = Object.fromEntries(Object.values(LANGUAGES).map(language => {
    const subset = rows.filter(row => row.language === language);
    return [language, { files: subset.length, passed: subset.filter(row => row.status === 'pass').length, max_ms: Math.max(0, ...subset.map(row => row.ms || 0)) }];
  }));
  console.log(JSON.stringify({ report, server: output.server, summary, error: output.error }, null, 2));
  process.exitCode = output.error || rows.some(row => row.status !== 'pass') ? 1 : 0;
}
main().catch(error => { console.error(error); process.exitCode = 1; });
