const { runTests, downloadAndUnzipVSCode } = require('@vscode/test-electron');
const { execFileSync, spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const VSCODE_VERSION = '1.142.0';
const EXTENSION = path.resolve(__dirname, '../..');
const ROOT = path.resolve(EXTENSION, '../..');
const SHARDS = ['examples', 'examples-fixed'].flatMap(root => ['mutated-state', 'shared-state', 'unstructured-state'].map(family => `${root}/${family}`));
const BASELINE = path.join(ROOT, 'tooling/state-lens/smoke.json');
const SERVER = process.env.STATE_LENS_SERVER || path.join(ROOT, 'tooling/state-lens/target/release/state-lens');
const { version } = require(path.join(EXTENSION, 'package.json'));
const VSIX = path.join(EXTENSION, `state-lens-${version}.vsix`);
const MERGER = path.join(ROOT, 'tooling/state-lens/scripts/smoke_report.py');

async function main() {
  let selected;
  let report = path.join(ROOT, 'build/smoke');
  let strict = false;
  const args = process.argv.slice(2);
  while (args.length) {
    const arg = args.shift();
    if (arg === '--shard') selected = args.shift();
    else if (arg === '--report') report = path.resolve(args.shift());
    else if (arg === '--strict-gaps') strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (selected && !SHARDS.includes(selected)) throw new Error(`Invalid shard: ${selected}`);
  const shards = selected ? [selected] : SHARDS;
  const jobs = Number(process.env.STATE_LENS_E2E_JOBS || Math.min(SHARDS.length, os.cpus().length));
  if (!Number.isInteger(jobs) || jobs < 1) throw new Error('STATE_LENS_E2E_JOBS must be positive');
  const scratch = fs.mkdtempSync(path.join(process.platform === 'darwin' ? '/tmp' : os.tmpdir(), 'sl-'));
  const inputs = [];
  let failed = false;
  try {
    for (const file of [VSIX, SERVER]) if (!fs.existsSync(file)) throw new Error(`${file} missing; run make server vscode`);
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
            folders: [{ path: path.join(ROOT, shard) }],
            settings: { 'stateLens.serverPath': SERVER, 'stateLens.diagnostics': true,
              'typescript.validate.enable': false, 'workbench.editor.enablePreview': false,
              'files.hotExit': 'off', 'window.restoreWindows': 'none' }
          }, null, 2));
          await runTests({
            vscodeExecutablePath: executable,
            extensionDevelopmentPath: path.join(directory, 'extension'),
            extensionTestsPath: path.join(__dirname, 'suite.cjs'),
            extensionTestsEnv: { STATE_LENS_ROOT: ROOT, STATE_LENS_BASELINE: BASELINE, STATE_LENS_SERVER: SERVER,
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
    const merged = spawnSync('python3', mergeArgs, { stdio: 'inherit' });
    failed ||= merged.status !== 0;
    fs.rmSync(scratch, { recursive: true, force: true });
  }
  process.exitCode = failed ? 1 : 0;
}
main().catch(error => { console.error(error.message || error); process.exitCode = 1; });
