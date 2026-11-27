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
  const script = `import * as fs from 'node:fs';
async function load(){
  let count=0; count+=1;
  const response=await fetch('/api'); const data=await response.text();
  const file=fs.openSync('x','r'); fs.closeSync(file);
  if(typeof data === 'string'){eval(data);}else{return count;}
}`;
  const sources = {
    typescript: script, javascript: script,
    python: `async def load():
    count=0
    count+=1
    data=input()
    file=open('x')
    file.close()
    if isinstance(data, str):
        eval(data)
    else:
        return count
`,
    go: `package main
import "net/http"
import "os"
func load(){count:=0; count+=1; data,err:=http.Get("url"); file,err:=os.Open("x"); defer file.Close(); go func(){}(); _,_,_=count,data,err;}`,
    rust: `async fn load(){let mut count=0; count+=1; let data=std::env::args(); let file=std::fs::File::open("x"); drop(file); let _=(count,data);}`
  };
  for (const [language, info] of Object.entries(registry)) fs.writeFileSync(path.join(workspace, `sample.${info.extensions[0]}`), sources[language]);
  await runTests({ vscodeExecutablePath: await downloadAndUnzipVSCode('1.142.0'), extensionDevelopmentPath: extensions, extensionTestsPath: path.join(__dirname, 'suite.cjs'), extensionTestsEnv: { STATE_LENS_ISOLATION_LANGUAGES: JSON.stringify(languages), STATE_LENS_ISOLATION_WORKSPACE: workspace }, launchArgs: [workspace, '--user-data-dir', path.join(scratch, 'profile'), '--extensions-dir', path.join(scratch, 'installed'), '--disable-workspace-trust', '--skip-welcome', '--skip-release-notes', '--disable-extension', 'github.copilot-chat'] });
}
main().catch(error => { console.error(error); process.exitCode = 1; });
