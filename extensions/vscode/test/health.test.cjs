const { test } = require('node:test');
const assert = require('node:assert/strict');
const { Client } = require('./health/client.cjs');

test('health client routes fragmented LSP responses and retains error notifications', async () => {
  const client = new Client(process.execPath, ['-e', `
    process.stdin.once('data', () => {
      const messages = [
        {jsonrpc:'2.0',method:'window/logMessage',params:{type:1,message:'unsupported language: python'}},
        {jsonrpc:'2.0',id:2,result:'second'},
        {jsonrpc:'2.0',id:1,error:{code:-32603,message:'document unavailable'}}
      ];
      const frames = Buffer.concat(messages.map(message => {
        const body = Buffer.from(JSON.stringify(message));
        return Buffer.concat([Buffer.from('Content-Length: '+body.length+'\\r\\n\\r\\n'),body]);
      }));
      process.stdout.write(frames.subarray(0, 13));
      setTimeout(() => process.stdout.write(frames.subarray(13)), 10);
    });
    setInterval(() => {}, 1000);
  `]);
  try {
    const first = assert.rejects(client.request('first', {}, 2000), /document unavailable/);
    const second = client.request('second', {}, 2000);
    assert.equal(await second, 'second');
    await first;
    assert.equal(client.logs[0].params.message, 'unsupported language: python');
  } finally { await client.stop(); }
});

test('health client bounds hung requests and releases pending requests', async () => {
  const client = new Client(process.execPath, ['-e', 'setInterval(() => {}, 1000)']);
  try {
    await assert.rejects(client.request('stateLens/ranges', {}, 50), /timeout after 50ms/);
    assert.equal(client.pending.size, 0);
  } finally { await client.stop(); }
});

test('health client reports server death and captures panic stderr', async () => {
  const client = new Client(process.execPath, ['-e', `process.stdin.once('data', () => { process.stderr.write('panic: fixture\\n'); process.exit(7); });`]);
  try {
    await assert.rejects(client.request('stateLens/ranges', {}, 2000), /server exited: code=7/);
    await client.closed;
    assert.match(client.logs.map(log => log.stderr || '').join(''), /panic: fixture/);
    assert.equal(client.pending.size, 0);
  } finally { await client.stop(); }
});
