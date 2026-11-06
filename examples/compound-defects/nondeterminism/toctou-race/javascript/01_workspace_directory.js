import { existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const PREFIX = "workspace-";
const DIRECTORY = "reports";
function gate() {
    let open = () => undefined;
    const promise = new Promise((resolve) => { open = resolve; });
    return { promise, open };
}
async function initialize(path, ready) {
    if (!existsSync(path)) {
        await ready;
        mkdirSync(path);
    }
}
async function main() {
    const directory = mkdtempSync(join(tmpdir(), PREFIX));
    const path = join(directory, DIRECTORY);
    try {
        const ready = gate();
        const pending = initialize(path, ready.promise);
        mkdirSync(path);
        ready.open();
        await pending;
    }
    catch (error) {
        console.log("directory", error.code);
    }
    finally {
        rmSync(directory, { recursive: true });
    }
}
void main();
