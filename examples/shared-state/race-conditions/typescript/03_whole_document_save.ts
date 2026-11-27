type Doc = { title: string; body: string };

const database: { doc: Doc } = { doc: { title: "Draft", body: "" } };

const tick = () => new Promise((resolve) => setTimeout(resolve, 1));

async function load(): Promise<Doc> {
  await tick();
  return { ...database.doc };
}

async function save(doc: Doc): Promise<void> {
  await tick();
  database.doc = doc;
}

async function editTitle() {
  const doc = await load();
  doc.title = "Final";
  await save(doc);
}

async function editBody() {
  const doc = await load();
  doc.body = "Hello";
  await save(doc);
}

async function main() {
  await Promise.all([editTitle(), editBody()]);
  console.log(database.doc);
}

main();
