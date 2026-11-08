const BODY = '{"colour":"blue"}';

const recordLoader = {
  load(this: Record<string, unknown>, body: string): void {
    const record: Record<string, unknown> = JSON.parse(body);
    for (const key of Object.keys(record)) {
      this[key] = record[key];
    }
  },
};

function main(): void {
  const paint: { color?: string } = {};
  const catalog = Object.assign(paint, recordLoader);
  catalog.load(BODY);
  console.log("paint", catalog.color.toUpperCase());
}

main();
