import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { constants, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { FAMILIES, WORK } from './ingredient-spec.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const EXECUTION_TIMEOUT_MS = 10_000;
const COMPILATION_TIMEOUT_MS = 60_000;
const TIMING_RUNS = 10;
const REQUIRED_SUCCESSES = 9;
const SIGNAL_EXIT_BASE = 128;
const LANGUAGES = ['python', 'typescript', 'go', 'rust'];
const EXTENSIONS = { python: 'py', typescript: 'ts', go: 'go', rust: 'rs' };
const TOTALS = { python: 43, typescript: 37, go: 44, rust: 44 };
const MEMORY_COMPOUNDS = new Set([
  'use-after-free', 'buffer-overflow', 'type-confusion-corruption',
  'torn-write', 'unchecked-sentinel-corruption',
]);
const DYNAMIC_COMPOUNDS = new Set([
  'type-confusion', 'shape-drift', 'aliased-default-type-confusion',
  'unexpected-type-element', 'stale-api-upgrade', 'deserialized-type-confusion',
  'schema-drift-on-load',
]);

function assessmentRows() {
  const rows = new Map();
  const pattern = /^\| \[([^\]]+)\]\(([^/]+)\/[^)]+\) \| ([\d, ]+) \| ([012]) \| ([012]) \| ([012]) \| ([012]) \|/gm;
  for (const match of readFileSync(join(ROOT, 'ASSESSMENT.md'), 'utf8').matchAll(pattern)) {
    const [, slug, className, cwes, ...scores] = match;
    rows.set(`${className}/${slug}`, {
      scores: Object.fromEntries(LANGUAGES.map((language, index) => [language, Number(scores[index])])),
      cwes: cwes.split(',').map(cwe => `CWE-${cwe.trim()}`),
    });
  }
  assert.equal(rows.size, 45, 'Assessment must contain all 45 compounds');
  return rows;
}

function discover(language) {
  const programs = [];
  const sources = [];
  for (const classDir of readdirSync(ROOT, { withFileTypes: true }).filter(entry => entry.isDirectory())) {
    for (const defectDir of readdirSync(join(ROOT, classDir.name), { withFileTypes: true }).filter(entry => entry.isDirectory())) {
      const directory = join(ROOT, classDir.name, defectDir.name);
      const languageDir = join(directory, language);
      if (existsSync(languageDir)) {
        for (const file of readdirSync(languageDir).filter(file => file.endsWith(`.${EXTENSIONS[language]}`))) {
          sources.push(join(languageDir, file));
        }
      }
      const manifestPath = join(directory, 'manifest.json');
      if (!existsSync(manifestPath)) continue;
      const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
      for (const example of manifest.examples.filter(example => example.file.startsWith(`${language}/`) || example.language === language)) {
        programs.push({ directory, className: classDir.name, slug: defectDir.name, manifest, example });
      }
    }
  }
  return { programs, sources };
}

function assertEscapeHatch(language, slug, source, mechanism) {
  let witnesses;
  if (language === 'typescript') {
    witnesses = [['SharedArrayBuffer', 'SharedArrayBuffer'], ['detach', 'detach'], ['>>> 0', '>>> 0']];
  } else if (language === 'python') {
    witnesses = slug === 'signedness-poisons-aggregate'
      ? [['ctypes', 'ctypes'], ['struct', 'struct']]
      : [['ctypes', 'ctypes']];
  } else if (MEMORY_COMPOUNDS.has(slug)) {
    witnesses = [['unsafe', 'unsafe']];
  } else if (DYNAMIC_COMPOUNDS.has(slug)) {
    witnesses = language === 'go'
      ? [['any', '\\bany\\b'], ['interface', 'interface\\s*\\{'], ['map', 'map\\[']]
      : [['Any', '\\bAny\\b'], ['downcast', 'downcast'], ['HashMap', 'HashMap'], ['enum', '\\benum\\b']];
  } else if (slug === 'integer-overflow-buffer') {
    assert.match(mechanism, /wrap/i, `${slug}: mechanism must explain the size wrap`);
    assert.match(mechanism, /bound|trap|panic/i, `${slug}: mechanism must explain the trapped access`);
    witnesses = language === 'go'
      ? [['uint', 'uint(?:8|16|32|64)']]
      : [['wrapping', 'wrapping_(?:add|mul)']];
  } else if (language === 'rust' && slug === 'null-dereference') {
    witnesses = [['Option', 'Option'], ['unwrap', '\\.unwrap\\(']];
  } else if (language === 'rust' && slug === 'lazy-init-visibility-race') {
    witnesses = [['relaxed', 'Ordering::Relaxed']];
  } else if (language === 'rust' && ['lost-update', 'concurrent-modification'].includes(slug)) {
    witnesses = [['Mutex', 'Mutex']];
  } else if (language === 'go' && slug === 'dispatch-order-divergence') {
    witnesses = [['registration', '(?:register|install)'], ['plugin', '(?:register|install)']];
  }
  assert.ok(witnesses, `${language}/${slug}: missing score-1 policy`);
  assert.ok(witnesses.some(([description, syntax]) =>
    mechanism.toLowerCase().includes(description.toLowerCase()) && new RegExp(syntax).test(source)),
  `${language}/${slug}: source and mechanism must identify its assessment-approved escape hatch or trapped form`);
}

function transcript(language, result) {
  const lines = result.stderr.split('\n');
  let error;
  if (language === 'rust') {
    const header = lines.findIndex(line => /^thread .* panicked at /.test(line));
    if (header >= 0) error = lines[header + 1];
  } else if (language === 'go') {
    error = lines.find(line => /^(panic:|fatal error:)/.test(line));
  } else {
    error = lines.find(line => /^[A-Za-z]*(?:Error|Exception)(?: \[[^\]]+\])?:/.test(line));
  }
  return [result.stdout.trimEnd(), error].filter(Boolean).join('\n');
}

function runner(language, file, temporary) {
  if (language === 'python') return ['python3', [file]];
  if (language === 'typescript') return [process.execPath, [file]];
  const binary = join(temporary, 'example');
  const command = language === 'go' ? 'go' : 'rustc';
  const args = language === 'go'
    ? ['build', '-o', binary, file]
    : ['--edition', '2021', '-o', binary, file];
  const compiled = spawnSync(command, args, {
    encoding: 'utf8', timeout: COMPILATION_TIMEOUT_MS,
    env: { ...process.env, GOCACHE: join(tmpdir(), 'compound-defects-go-cache'), RUST_BACKTRACE: '0' },
  });
  assert.ok(!compiled.error && compiled.status === 0,
    `${file}: compilation failed: ${compiled.error?.message ?? ''}\n${compiled.stdout}${compiled.stderr}`);
  return [binary, []];
}

export function testLanguage(language) {
  const rows = assessmentRows();
  const { programs, sources } = discover(language);
  test(`${language}: complete coverage, no score-zero files, and no unmanifested programs`, () => {
    const expected = [...rows].filter(([, row]) => row.scores[language] > 0).map(([key]) => key);
    assert.equal(expected.length, TOTALS[language]);
    assert.deepEqual(programs.map(({ className, slug }) => `${className}/${slug}`).sort(), expected.sort());
    assert.deepEqual(sources.sort(), programs.map(({ directory, example }) => join(directory, example.file)).sort());
    assert.deepEqual(Object.entries(WORK).flatMap(([className, compounds]) =>
      Object.keys(compounds).map(slug => `${className}/${slug}`)).sort(), [...rows.keys()].sort());
  });

  for (const { directory, className, slug, manifest, example } of programs) {
    test(`${language}/${className}/${slug}: both ingredients and recorded behavior`, () => {
      const expected = WORK[className]?.[slug];
      const ingredients = example.ingredients;
      const slugs = (expected ?? []).map(ingredient => ingredient.split('/')[1]);
      const descriptions = (Array.isArray(ingredients) ? ingredients : []).map(ingredient => ingredient?.suggests).join(' ');
      const context = `${example.file} does not suggest ${slugs.map(slug => `\`${slug}\``).join(' and ')}. ${descriptions}`;
      assert.ok(expected, context);
      assert.ok(Array.isArray(ingredients), context);
      assert.equal(ingredients.length, 2, context);
      assert.ok(ingredients.every(ingredient => ingredient && typeof ingredient === 'object'), context);
      assert.deepEqual(ingredients.map(ingredient => ingredient.family).sort(), [...FAMILIES[className]].sort(), context);
      assert.deepEqual(ingredients.map(ingredient => `${ingredient.family}/${ingredient.defect}`).sort(), [...expected].sort(), context);
      assert.equal(manifest.family, 'compound-defects');
      assert.equal(manifest.class, className);
      assert.equal(manifest.defect, slug);
      assert.equal(example.language, language);
      assert.match(example.file, new RegExp(`^${language}/01_[a-z0-9_]+\\.${EXTENSIONS[language]}$`));
      const row = rows.get(`${className}/${slug}`);
      assert.deepEqual(manifest.scores, row.scores);
      assert.deepEqual(manifest.cwes, row.cwes);
      const file = join(directory, example.file);
      const source = readFileSync(file, 'utf8');
      assert.doesNotMatch(source, /\b(?:bug|CWE|race)\b/i, 'Source must remain blind');
      assert.ok(!source.includes(slug), 'Source must not name its compound defect');
      const labels = new Set(['bug', ...Object.values(FAMILIES).flat(), ...Object.values(WORK).flatMap(compounds =>
        Object.values(compounds).flat().map(ingredient => ingredient.split('/')[1]))]);
      for (const ingredient of ingredients) {
        assert.ok(typeof ingredient.suggests === 'string' && ingredient.suggests.trim(), context);
        assert.ok(Array.isArray(ingredient.evidence) && ingredient.evidence.length > 0, context);
        for (const evidence of ingredient.evidence) {
          assert.ok(typeof evidence === 'string' && evidence.trim(), context);
          assert.ok(!labels.has(evidence.trim().toLowerCase()), `${context} Evidence must be source, not a label.`);
          assert.ok(source.includes(evidence), `${context} Missing evidence: ${JSON.stringify(evidence)}`);
        }
      }
      const score = manifest.scores[language];
      assert.ok(score === 1 || score === 2, `${slug}: score ${score} must not have a ${language} file`);
      for (const field of ['summary', 'mechanism', 'expected_behavior', 'observed_output']) {
        assert.ok(typeof example[field] === 'string' && example[field].trim(), `${slug}: missing ${field}`);
      }
      if (score === 1) assertEscapeHatch(language, slug, source, example.mechanism);
      assert.ok(Array.isArray(example.bug_lines) && example.bug_lines.length > 0);
      for (const line of example.bug_lines) {
        assert.ok(Number.isInteger(line) && line > 0 && line <= source.trimEnd().split('\n').length);
      }
      assert.equal(typeof example.deterministic, 'boolean');
      assert.ok(Number.isInteger(example.exit_code));
      const temporary = mkdtempSync(join(tmpdir(), 'compound-example-'));
      try {
        const [command, args] = runner(language, file, temporary);
        const runs = example.deterministic ? 1 : TIMING_RUNS;
        const outcomes = Array.from({ length: runs }, () => {
          const result = spawnSync(command, args, {
            encoding: 'utf8', timeout: EXECUTION_TIMEOUT_MS, cwd: temporary,
            env: { ...process.env, RUST_BACKTRACE: '0', PYTHONDONTWRITEBYTECODE: '1' },
          });
          const combined = result.stdout + result.stderr;
          const status = result.signal ? SIGNAL_EXIT_BASE + constants.signals[result.signal] : result.status;
          const matches = !result.error && status === example.exit_code
            && (combined.includes(example.observed_output) || transcript(language, result).includes(example.observed_output));
          return { matches, status, signal: result.signal, error: result.error?.message, combined };
        });
        const required = example.deterministic ? 1 : REQUIRED_SUCCESSES;
        assert.ok(outcomes.filter(outcome => outcome.matches).length >= required,
          `${slug}: expected exit ${example.exit_code} and ${JSON.stringify(example.observed_output)} in ${required}/${runs} runs.\n${JSON.stringify(outcomes, null, 2)}`);
      } finally {
        rmSync(temporary, { recursive: true, force: true });
      }
    });
  }
}
