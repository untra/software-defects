const database = { doc: { title: "Draft", body: "" } };
const tick = () => new Promise((resolve) => setTimeout(resolve, 1));
async function load() {
    await tick();
    return { ...database.doc };
}
async function save(doc) {
    await tick();
    database.doc = { ...database.doc, ...doc };
}
async function editTitle() {
    const doc = await load();
    doc.title = "Final";
    await save({ title: doc.title });
}
async function editBody() {
    const doc = await load();
    doc.body = "Hello";
    await save({ body: doc.body });
}
async function main() {
    await Promise.all([editTitle(), editBody()]);
    console.log(database.doc);
}
main();
