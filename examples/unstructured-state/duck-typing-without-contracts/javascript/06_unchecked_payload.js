function titles(payload) {
    const records = payload;
    return records.map((record) => record.title.toUpperCase());
}
console.log(titles(JSON.parse('{"title":"invoice"}')));
