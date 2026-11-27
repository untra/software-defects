const DEFAULT_HEADING = "Public catalog";
const pages = new Map<string, string>();

type Request = { path: string; query?: { heading?: string } };

function render(request: Request): string {
  const cached = pages.get(request.path);
  if (cached !== undefined) return cached;
  const body = request.query?.heading ?? DEFAULT_HEADING;
  pages.set(request.path, body);
  return body;
}

function main(): void {
  const CATALOG_PATH = "/catalog";
  render({ path: CATALOG_PATH, query: { heading: "Members only" } });
  console.log("public page", render({ path: CATALOG_PATH }));
}

main();
