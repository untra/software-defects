const POOL_SIZE = 3;
const WATCHDOG_MS = 200;

class ConnectionPool {
  private free = POOL_SIZE;
  private waiters: Array<() => void> = [];
  async acquire(): Promise<void> {
    if (this.free > 0) {
      this.free--;
      return;
    }
    await new Promise<void>((resolve) => this.waiters.push(resolve));
  }
  release(): void {
    const next = this.waiters.shift();
    if (next) next();
    else this.free++;
  }
}

const pool = new ConnectionPool();

async function query(sql: string): Promise<string> {
  await pool.acquire();
  try {
    if (sql.includes("bad")) throw new Error("syntax error");
    return `rows for ${sql}`;
  } finally { pool.release(); }
}

async function main() {
  for (let i = 0; i < POOL_SIZE; i++) await query("select bad").catch(() => {});
  const timer = setTimeout(() => {
    console.log(`healthy query still waiting for a connection after ${WATCHDOG_MS}ms`);
    process.exit(0);
  }, WATCHDOG_MS);
  console.log(await query("select 1"));
  clearTimeout(timer);
}

main();
