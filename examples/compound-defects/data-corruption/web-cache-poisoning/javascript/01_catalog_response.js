const DEFAULT_HEADING = "Public catalog";
const pages = new Map();
function render(request) {
    const cached = pages.get(request.path);
    if (cached !== undefined)
        return cached;
    const body = request.query?.heading ?? DEFAULT_HEADING;
    pages.set(request.path, body);
    return body;
}
function main() {
    const CATALOG_PATH = "/catalog";
    render({ path: CATALOG_PATH, query: { heading: "Members only" } });
    console.log("public page", render({ path: CATALOG_PATH }));
}
main();
