const { test } = require('node:test');
const assert = require('node:assert/strict');
const { ServerLifecycle } = require('../dist/lifecycle');
const deferred = () => { let resolve; const promise = new Promise(done => { resolve = done; }); return { promise, resolve }; };
function harness(start = async () => {}) {
  const calls = [];
  const lifecycle = new ServerLifecycle({ start: async () => { calls.push('start'); await start(); }, stop: async () => { calls.push('stop'); }, invalidate: () => calls.push('invalidate'), changed: state => calls.push(state), error: () => calls.push('error') });
  return { lifecycle, calls };
}
test('hidden documents do not start a server and repeated visibility does not restart it', async () => {
  const { lifecycle, calls } = harness();
  await lifecycle.update(false);
  assert.equal(calls.length, 0);
  await lifecycle.update(true);
  await lifecycle.update(true);
  assert.equal(calls.filter(c => c === 'start').length, 1);
  await lifecycle.update(false);
  assert.equal(lifecycle.state, 'idle');
  assert.equal(calls.filter(c => c === 'stop').length, 1);
});
test('a startup finishing after hide/show is stopped before a fresh startup', async () => {
  const gate = deferred();
  const { lifecycle, calls } = harness(() => gate.promise);
  const pending = lifecycle.update(true);
  await Promise.resolve();
  void lifecycle.update(false);
  void lifecycle.update(true);
  gate.resolve(); await pending;
  assert.deepEqual(calls.filter(c => ['start', 'stop'].includes(c)), ['start', 'stop', 'start']);
  assert.equal(lifecycle.state, 'running');
});
test('shutdown during startup cannot leave a server running', async () => {
  const gate = deferred();
  const { lifecycle } = harness(() => gate.promise);
  void lifecycle.update(true);
  await Promise.resolve();
  const pending = lifecycle.update(false);
  gate.resolve(); await pending;
  assert.equal(lifecycle.state, 'idle');
});
test('failed startup waits for explicit retry or visibility transition', async () => {
  const { lifecycle, calls } = harness(async () => { throw Error('missing binary'); });
  await lifecycle.update(true);
  await lifecycle.update(true);
  assert.equal(lifecycle.state, 'failed');
  assert.equal(calls.filter(c => c === 'start').length, 1);
  await lifecycle.update(true, true);
  assert.equal(calls.filter(c => c === 'start').length, 2);
});
test('a crashed server stays failed until requested retry', async () => {
  const { lifecycle, calls } = harness();
  await lifecycle.update(true);
  lifecycle.crashed();
  await lifecycle.update(true);
  assert.equal(lifecycle.state, 'failed');
  assert.equal(calls.filter(c => c === 'start').length, 1);
  await lifecycle.update(true, true);
  assert.equal(lifecycle.state, 'running');
});
test('visibility updates in the same event turn are not lost', async () => {
  const { lifecycle } = harness();
  void lifecycle.update(false);
  await lifecycle.update(true);
  assert.equal(lifecycle.state, 'running');
});
