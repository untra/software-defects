const DEFAULT_PAGE_SIZE = 20;
const MAX_PAGE_SIZE = 100;

type Query = { table: string; pageSize?: number };

function buildQueries(queries: Query[]): string[] {
  const sql: string[] = [];
  for (const q of queries) {
    let pageSize = DEFAULT_PAGE_SIZE;
    if (q.pageSize !== undefined) pageSize = Math.min(q.pageSize, MAX_PAGE_SIZE);
    sql.push(`SELECT * FROM ${q.table} LIMIT ${pageSize}`);
  }
  return sql;
}

console.log(
  buildQueries([{ table: "users" }, { table: "orders", pageSize: 50 }, { table: "items" }]).join("\n"),
);
