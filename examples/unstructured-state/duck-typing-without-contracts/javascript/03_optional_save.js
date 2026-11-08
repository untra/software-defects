function persist(doc) {
    return doc.save();
}
console.log(persist({}));
