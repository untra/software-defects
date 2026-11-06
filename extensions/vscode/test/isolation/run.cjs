const { runTests, downloadAndUnzipVSCode } = require('@vscode/test-electron');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const root = path.resolve(__dirname, '../../../..');
const registry = require('../../languages.json');
const { version } = require('../../package.json');
async function main() {
  const languages = process.argv[2] ? [process.argv[2]] : Object.keys(registry);
  if (languages.some(language => !registry[language])) throw new Error('Unknown language');
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'state-lens-isolation-'));
  const extensions = languages.map(language => {
    const destination = path.join(scratch, language);
    const vsix = path.join(root, 'build/vsix', `state-lens-${language}-${version}-${process.platform}-${process.arch}.vsix`);
    execFileSync(process.platform === 'win32' ? 'python' : 'python3', ['-m', 'zipfile', '-e', vsix, destination]);
    fs.chmodSync(path.join(destination, 'extension/bin', process.platform === 'win32' ? 'state-lens.exe' : 'state-lens'), 0o755);
    return path.join(destination, 'extension');
  });
  const workspace = path.join(scratch, 'workspace'); fs.mkdirSync(workspace);
  const sources = { typescript: 'function clock(){return Date.now();}', javascript: 'function clock(){return Date.now();}', python: 'import time\ndef clock():\n    return time.time()\n', go: 'package main\nimport "time"\nfunc clock() int64 { return time.Now().Unix() }', rust: 'fn clock() { let _ = std::time::SystemTime::now(); }' };
  for (const [language, info] of Object.entries(registry)) fs.writeFileSync(path.join(workspace, `sample.${info.extensions[0]}`), sources[language]);
  await runTests({ vscodeExecutablePath: await downloadAndUnzipVSCode('1.142.0'), extensionDevelopmentPath: extensions, extensionTestsPath: path.join(__dirname, 'suite.cjs'), extensionTestsEnv: { STATE_LENS_ISOLATION_LANGUAGES: JSON.stringify(languages), STATE_LENS_ISOLATION_WORKSPACE: workspace }, launchArgs: [workspace, '--user-data-dir', path.join(scratch, 'profile'), '--extensions-dir', path.join(scratch, 'installed'), '--disable-workspace-trust', '--skip-welcome', '--skip-release-notes', '--disable-extension', 'github.copilot-chat'] });
}
main().catch(error => { console.error(error); process.exitCode = 1; });
