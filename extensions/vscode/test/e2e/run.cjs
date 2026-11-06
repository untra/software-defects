const { runTests, downloadAndUnzipVSCode } = require('@vscode/test-electron');
const { execFileSync, spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const VSCODE_VERSION = '1.142.0';
const EXTENSION = path.resolve(__dirname, '../..');
const ROOT = path.resolve(EXTENSION, '../..');
const SHARDS = JSON.parse(execFileSync('python3', [path.join(ROOT, 'tooling/state-lens/scripts/corpus_inventory.py')], { encoding: 'utf8' }));
const BASELINE = path.join(ROOT, 'tooling/state-lens/smoke.json');
let server = process.env.STATE_LENS_SERVER || path.join(ROOT, 'tooling/state-lens/target/release/state-lens');
const { version } = require(path.join(EXTENSION, 'package.json'));
const VSIX = path.join(EXTENSION, `state-lens-${version}.vsix`);
const MERGER = path.join(ROOT, 'tooling/state-lens/scripts/smoke_report.py');

async function main() {
  let selected;
  let report = path.join(ROOT, 'build/smoke');
  let strict = false;
  let health = false;
  let file;
  let language;
  let workspaceRoot;
  const args = process.argv.slice(2);
  while (args.length) {
    const arg = args.shift();
    if (arg === '--shard') selected = args.shift();
    else if (arg === '--report') report = path.resolve(args.shift());
    else if (arg === '--strict-gaps') strict = true;
    else if (arg === '--health-only') health = true;
    else if (arg === '--server') server = path.resolve(args.shift());
    else if (arg === '--file') file = path.relative(ROOT, path.resolve(args.shift())).split(path.sep).join('/');
    else if (arg === '--language') language = args.shift();
    else if (arg === '--workspace') workspaceRoot = path.resolve(args.shift());
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (selected && !SHARDS.includes(selected)) throw new Error(`Invalid shard: ${selected}`);
  if (file && !health || language && !health) throw new Error('--file and --language require --health-only');
  if (language && !['go', 'rust', 'typescript', 'python'].includes(language)) throw new Error(`Invalid language: ${language}`);
  const shards = (selected ? [selected] : SHARDS).filter(shard => !file || file.startsWith(shard + '/'));
  if (!shards.length) throw new Error('No matching corpus shards');
  const jobs = Number(process.env.STATE_LENS_E2E_JOBS || Math.min(SHARDS.length, os.cpus().length));
  if (!Number.isInteger(jobs) || jobs < 1) throw new Error('STATE_LENS_E2E_JOBS must be positive');
  const scratch = fs.mkdtempSync(path.join(process.platform === 'darwin' ? '/tmp' : os.tmpdir(), 'sl-'));
  const inputs = [];
  let failed = false;
  try {
    for (const file of [VSIX, server]) if (!fs.existsSync(file)) throw new Error(`${file} missing; run make server vscode`);
    const executable = await downloadAndUnzipVSCode(VSCODE_VERSION);
    let next = 0;
    await Promise.all(Array.from({ length: Math.min(jobs, shards.length) }, async () => {
      while (next < shards.length) {
        const shard = shards[next++];
        const directory = path.join(scratch, shard.replace('/', '-'));
        fs.mkdirSync(directory, { recursive: true });
        const shardReport = path.join(directory, 'smoke-report.json');
        try {
          execFileSync('unzip', ['-q', VSIX, 'extension/*', '-d', directory]);
          const workspace = path.join(directory, 'corpus.code-workspace');
          fs.writeFileSync(workspace, JSON.stringify({
            folders: [{ path: workspaceRoot || path.join(ROOT, shard) }],
            settings: { 'stateLens.serverPath': server, 'stateLens.diagnostics': true,
              'typescript.validate.enable': false, 'workbench.editor.enablePreview': false,
              'files.hotExit': 'off', 'window.restoreWindows': 'none' }
          }, null, 2));
          await runTests({
            vscodeExecutablePath: executable,
            extensionDevelopmentPath: path.join(directory, 'extension'),
            extensionTestsPath: path.join(__dirname, 'suite.cjs'),
            extensionTestsEnv: { STATE_LENS_ROOT: ROOT, STATE_LENS_BASELINE: BASELINE, STATE_LENS_SERVER: server,
              STATE_LENS_HEALTH_ONLY: health ? '1' : '0', STATE_LENS_FILE: file || '', STATE_LENS_LANGUAGE: language || '',
              STATE_LENS_SHARD: shard, STATE_LENS_REPORT: shardReport, STATE_LENS_STRICT_GAPS: strict ? '1' : '0' },
            launchArgs: [workspace, '--disable-extensions', '--disable-workspace-trust', '--skip-welcome', '--skip-release-notes',
              '--user-data-dir', path.join(directory, 'user'), '--extensions-dir', path.join(directory, 'extensions')]
          });
        } catch (error) {
          failed = true;
          console.error(`${shard}: ${error.message || error}`);
          if (fs.existsSync(shardReport)) {
            const payload = JSON.parse(fs.readFileSync(shardReport, 'utf8'));
            if (!payload.rows.some(row => row.status === 'fail') && !(strict && payload.rows.some(row => row.status === 'gap'))) {
              for (const row of payload.rows) {
                row.status = 'fail';
                row.problems.push({ category: 'crash', message: String(error.message || error) });
              }
              fs.writeFileSync(shardReport, JSON.stringify(payload));
            }
          }
        } finally {
          if (fs.existsSync(shardReport)) {
            const destination = path.join(report, 'editor-shards', shard, 'smoke-report.json');
            fs.mkdirSync(path.dirname(destination), { recursive: true });
            fs.copyFileSync(shardReport, destination);
            inputs.push(destination);
          }
          const logs = path.join(directory, 'user/logs');
          if (fs.existsSync(logs)) fs.cpSync(logs, path.join(report, 'editor-shards', shard, 'logs'), { recursive: true });
        }
      }
    }));
  } catch (error) {
    failed = true;
    console.error(error.message || error);
  } finally {
    const mergeArgs = [MERGER, ...inputs, '--report', report, '--layers', 'editor'];
    if (selected) mergeArgs.push('--shard', selected);
    if (strict) mergeArgs.push('--strict-gaps');
    if (health) {
      const rows = inputs.flatMap(input => JSON.parse(fs.readFileSync(input, 'utf8')).rows);
      fs.mkdirSync(report, { recursive: true });
      fs.writeFileSync(path.join(report, 'health-report.json'), JSON.stringify({ mode: 'editor-health', server, rows }, null, 2));
      failed ||= !rows.length || rows.some(row => row.status !== 'pass');
    } else {
      const merged = spawnSync('python3', mergeArgs, { stdio: 'inherit' });
      failed ||= merged.status !== 0;
    }
    fs.rmSync(scratch, { recursive: true, force: true });
  }
  process.exitCode = failed ? 1 : 0;
}
main().catch(error => { console.error(error.message || error); process.exitCode = 1; });
