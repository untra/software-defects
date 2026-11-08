type Doc = { save?: () => string };

function persist(doc: Doc): string {
  return doc.save ? doc.save() : "document cannot be saved";
}

console.log(persist({}));
