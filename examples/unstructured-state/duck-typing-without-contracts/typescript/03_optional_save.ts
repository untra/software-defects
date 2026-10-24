type Doc = { save?: () => string };

function persist(doc: Doc): string {
  return doc.save();
}

console.log(persist({}));
