const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const esbuild = require('esbuild');
const languages = require('../languages.json');
const extension = path.resolve(__dirname, '..');
const root = path.resolve(extension, '../..');
const targets = {
  'linux-x64': 'x86_64-unknown-linux-gnu', 'linux-arm64': 'aarch64-unknown-linux-gnu',
  'darwin-x64': 'x86_64-apple-darwin', 'darwin-arm64': 'aarch64-apple-darwin',
  'win32-x64': 'x86_64-pc-windows-msvc', 'win32-arm64': 'aarch64-pc-windows-msvc',
};
async function main() {
  const [language, target = `${process.platform}-${process.arch}`] = process.argv.slice(2);
  if (!languages[language] || !targets[target]) throw new Error('Usage: package.cjs <language> [platform-architecture]');
  const identity = `state-lens-${language}`;
  const namespace = `stateLens.${language}`;
  const cargoTarget = path.join(root, 'build', 'servers', language, target);
  execFileSync('cargo', ['build', '--release', '--locked', '--manifest-path', path.join(root, 'tooling/state-lens/Cargo.toml'), '--no-default-features', '--features', language, '--target', targets[target], '--target-dir', cargoTarget], { stdio: 'inherit' });
  const stage = fs.mkdtempSync(path.join(require('node:os').tmpdir(), `${identity}-`));
  const manifest = JSON.parse(fs.readFileSync(path.join(extension, 'package.json')));
  manifest.files = ['dist/extension.js', 'bin/*', 'README.md', 'LICENSE'];
  manifest.name = identity;
  manifest.displayName = `State Lens: ${languages[language].label}`;
  manifest.description = `State Lens analysis for ${languages[language].label}, with a bundled language server.`;
  manifest.activationEvents = [`onLanguage:${language}`];
  manifest.contributes = JSON.parse(JSON.stringify(manifest.contributes).replaceAll('stateLens.', `${namespace}.`).replaceAll('State Lens', manifest.displayName));
  for (const views of Object.values(manifest.contributes.views)) for (const view of views) { view.name = 'State Lens Evidence'; view.when = `${namespace}.activeEditor`; }
  manifest.contributes.configuration.properties[`${namespace}.serverPath`].description = 'Absolute development override. Empty uses the bundled language server.';
  delete manifest.scripts; delete manifest.dependencies; delete manifest.devDependencies;
  fs.writeFileSync(path.join(stage, 'package.json'), JSON.stringify(manifest, null, 2));
  fs.mkdirSync(path.join(stage, 'bin'));
  const binary = target.startsWith('win32') ? 'state-lens.exe' : 'state-lens';
  fs.copyFileSync(path.join(cargoTarget, targets[target], 'release', binary), path.join(stage, 'bin', binary));
  fs.chmodSync(path.join(stage, 'bin', binary), 0o755);
  for (const name of ['README.md', 'LICENSE']) fs.copyFileSync(path.join(extension, name), path.join(stage, name));
  await esbuild.build({ entryPoints: [path.join(extension, 'src/extension.ts')], bundle: true, platform: 'node', format: 'cjs', target: 'node20', external: ['vscode'], outfile: path.join(stage, 'dist/extension.js'), define: { STATE_LENS_LANGUAGE: JSON.stringify(language), STATE_LENS_EXTENSIONS: JSON.stringify(languages[language].extensions) } });
  const output = path.join(root, 'build', 'vsix'); fs.mkdirSync(output, { recursive: true });
  const vsix = path.join(output, `${identity}-${manifest.version}-${target}.vsix`);
  execFileSync(process.execPath, [require.resolve('@vscode/vsce/vsce'), 'package', '--no-dependencies', '--target', target, '--out', vsix], { cwd: stage, stdio: 'inherit' });
  console.log(vsix);
}
main().catch(error => { console.error(error); process.exitCode = 1; });
