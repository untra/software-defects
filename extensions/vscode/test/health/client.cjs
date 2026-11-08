const { spawn } = require('node:child_process');

const LOG_LIMIT = 200;
const FRAME_LIMIT = 64 * 1024 * 1024;
class Client {
  constructor(command, args = ['serve']) {
    this.logs = [];
    this.logCount = 0;
    this.pending = new Map();
    this.nextId = 0;
    this.buffer = Buffer.alloc(0);
    this.process = spawn(command, args, { env: { ...process.env, RUST_BACKTRACE: '1' }, stdio: 'pipe' });
    this.closed = new Promise(resolve => this.process.once('close', resolve));
    this.process.on('error', error => this.fail(error));
    this.process.on('exit', (code, signal) => this.fail(new Error(`server exited: code=${code} signal=${signal}`)));
    this.process.stdin.on('error', error => this.fail(error));
    this.process.stderr.on('data', data => this.record({ stderr: data.toString() }));
    this.process.stdout.on('data', data => {
      try {
        this.buffer = Buffer.concat([this.buffer, data]);
        while (this.buffer.length) {
          const end = this.buffer.indexOf('\r\n\r\n');
          if (end < 0) {
            if (this.buffer.length > FRAME_LIMIT) throw new Error('invalid LSP header');
            break;
          }
          const length = Number(this.buffer.subarray(0, end).toString().match(/Content-Length: (\d+)/i)?.[1]);
          if (!Number.isSafeInteger(length) || length < 0 || length > FRAME_LIMIT) throw new Error('invalid LSP frame length');
          if (this.buffer.length < end + 4 + length) break;
          const message = JSON.parse(this.buffer.subarray(end + 4, end + 4 + length));
          this.buffer = this.buffer.subarray(end + 4 + length);
          if (message.id !== undefined) {
            const request = this.pending.get(message.id);
            if (!request) continue;
            this.pending.delete(message.id);
            clearTimeout(request.timer);
            if (message.error) request.reject(Object.assign(new Error(message.error.message), { code: message.error.code }));
            else request.resolve(message.result);
          } else if (message.method === 'window/logMessage' || message.method === 'window/showMessage') this.record(message);
        }
      } catch (error) { this.fail(error); }
    });
  }
  record(value) {
    this.logs.push(value);
    this.logCount++;
    if (this.logs.length > LOG_LIMIT) this.logs.shift();
  }
  logsSince(count) { return this.logs.slice(Math.max(0, count - (this.logCount - this.logs.length))); }
  fail(error) {
    this.failure = error;
    for (const request of this.pending.values()) {
      clearTimeout(request.timer);
      request.reject(error);
    }
    this.pending.clear();
  }
  send(message) {
    if (this.failure) throw this.failure;
    const body = Buffer.from(JSON.stringify({ jsonrpc: '2.0', ...message }));
    this.process.stdin.write(Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]));
  }
  notify(method, params) { this.send({ method, params }); }
  request(method, params, timeout) {
    if (this.failure) return Promise.reject(this.failure);
    const id = ++this.nextId;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`timeout after ${timeout}ms: ${method}`));
      }, timeout);
      this.pending.set(id, { resolve, reject, timer });
      try { this.send({ id, method, params }); }
      catch (error) { this.fail(error); }
    });
  }
  async stop() {
    this.fail(new Error('client closed'));
    this.process.kill('SIGKILL');
    await this.closed;
  }
}
module.exports = { Client };
