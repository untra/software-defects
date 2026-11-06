const { execFileSync } = require('node:child_process');
const path = require('node:path');
const root = path.resolve(__dirname, '../../..');
execFileSync('cargo', ['build', '--release', '--locked', '--manifest-path', path.join(root, 'tooling/state-lens/Cargo.toml')], { stdio: 'inherit' });
const server = path.join(root, 'tooling/state-lens/target/release', process.platform === 'win32' ? 'state-lens.exe' : 'state-lens');
console.log(`Built separate server: ${server}\nSet stateLens.serverPath to this absolute path, or install this executable on PATH. Packaging does not update your installed server.`);
