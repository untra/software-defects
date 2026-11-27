"""Validate the actual packaged payload and execute its native server."""
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[3]
language, target = sys.argv[1:]
version = json.loads((ROOT / 'extensions/vscode/package.json').read_text())['version']
archive = ROOT / 'build/vsix' / f'state-lens-{language}-{version}-{target}.vsix'
binary = 'state-lens.exe' if target.startswith('win32') else 'state-lens'
allowed = {'extension/package.json', 'extension/readme.md', 'extension/license.txt',
           'extension/dist/extension.js', f'extension/bin/{binary}',
           '[content_types].xml', 'extension.vsixmanifest'}
with zipfile.ZipFile(archive) as package:
    names = {name.lower() for name in package.namelist() if not name.endswith('/')}
    assert names == allowed, names.symmetric_difference(allowed)
    manifest = json.loads(package.read('extension/package.json'))
    assert manifest['name'] == f'state-lens-{language}'
    assert manifest['activationEvents'] == [f'onLanguage:{language}']
    assert all(key.startswith(f'stateLens.{language}.') for key in manifest['contributes']['configuration']['properties'])
    assert all(item['command'].startswith(f'stateLens.{language}.') for item in manifest['contributes']['commands'])
    assert all(view['name'] == 'State Lens Evidence' and view['when'] == f'stateLens.{language}.activeEditor' and view['id'].startswith(f'stateLens.{language}.') for views in manifest['contributes']['views'].values() for view in views)
    assert f'TargetPlatform="{target}"' in package.read('extension.vsixmanifest').decode()
    with tempfile.TemporaryDirectory() as directory:
        executable = Path(directory) / binary
        payload = package.read(f'extension/bin/{binary}')
        architecture = target.split('-')[1]
        if target.startswith('darwin'):
            assert payload[:4] == bytes.fromhex('cffaedfe')
            machine = struct.unpack_from('<I', payload, 4)[0]
            assert machine == {'x64': 0x01000007, 'arm64': 0x0100000C}[architecture]
        elif target.startswith('linux'):
            assert payload[:4] == b'\x7fELF'
            machine = struct.unpack_from('<H', payload, 18)[0]
            assert machine == {'x64': 62, 'arm64': 183}[architecture]
        else:
            assert payload[:2] == b'MZ'
            header = struct.unpack_from('<I', payload, 60)[0]
            assert payload[header:header + 4] == b'PE\0\0'
            machine = struct.unpack_from('<H', payload, header + 4)[0]
            assert machine == {'x64': 0x8664, 'arm64': 0xAA64}[architecture]
        executable.write_bytes(payload)
        executable.chmod(0o755)
        capabilities = json.loads(subprocess.check_output([str(executable), 'capabilities']))
        assert capabilities['languages'] == [language], capabilities
        subprocess.run([str(executable), 'check-queries'], check=True)
        samples = {'ts': 'function clock(){return Date.now();}', 'js': 'function clock(){return Date.now();}',
                   'go': 'package main\nfunc value() int { return 1 }', 'rs': 'fn value() -> i32 { 1 }',
                   'py': 'def value():\n    return 1\n'}
        for extension, source in samples.items():
            (Path(directory) / f'sample.{extension}').write_text(source)
        report = json.loads(subprocess.check_output([str(executable), 'scan', directory]))
        assert len(report['documents']) == 1, report
        assert report['documents'][0]['language'] == language, report
        print(json.dumps({'language': language, 'target': target, 'vsix_bytes': archive.stat().st_size,
                          'server_bytes': executable.stat().st_size}))

tree = subprocess.check_output(['cargo', 'tree', '--locked', '--manifest-path', str(ROOT / 'tooling/state-lens/Cargo.toml'), '--no-default-features', '--features', language, '--prefix', 'none'], text=True)
grammars = {line.split()[0] for line in tree.splitlines() if line.startswith('tree-sitter-') and not line.startswith('tree-sitter-language ')}
assert grammars == {f'tree-sitter-{language}'}, grammars
