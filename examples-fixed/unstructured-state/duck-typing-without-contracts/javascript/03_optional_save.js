function persist(doc) {
    return typeof doc?.save === "function" ? doc.save() : "document cannot be saved";
}
console.log(persist({}));
