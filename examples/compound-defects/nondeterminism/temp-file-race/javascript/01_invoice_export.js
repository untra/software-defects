import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const PREFIX = "invoice-";
const STAMP = 1700000000000;
const OTHER_DOCUMENT = "harbor invoice";
const DOCUMENT = "orchard invoice";
function gate() {
    let open = () => undefined;
    const promise = new Promise((resolve) => { open = resolve; });
    return { promise, open };
}
async function save(path, ready) {
    if (!existsSync(path)) {
        await ready;
        writeFileSync(path, DOCUMENT);
    }
}
async function main() {
    const directory = mkdtempSync(join(tmpdir(), PREFIX));
    const path = join(directory, PREFIX + STAMP);
    try {
        const ready = gate();
        const pending = save(path, ready.promise);
        writeFileSync(path, OTHER_DOCUMENT);
        ready.open();
        await pending;
        console.log("existing document", readFileSync(path, "utf8"));
    }
    finally {
        rmSync(directory, { recursive: true });
    }
}
void main();
