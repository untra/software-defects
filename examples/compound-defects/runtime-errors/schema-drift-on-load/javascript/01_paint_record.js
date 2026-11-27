const BODY = '{"colour":"blue"}';
const recordLoader = {
    load(body) {
        const record = JSON.parse(body);
        for (const key of Object.keys(record)) {
            this[key] = record[key];
        }
    },
};
function main() {
    const paint = {};
    const catalog = Object.assign(paint, recordLoader);
    catalog.load(BODY);
    console.log("paint", catalog.color.toUpperCase());
}
main();
