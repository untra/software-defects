function titles(payload: unknown): string[] {
    if (!Array.isArray(payload) || !payload.every(record =>
        record !== null && typeof record === "object" && typeof record.title === "string")) {
        return ["invalid records"];
    }
    return payload.map(record => record.title.toUpperCase());
}
console.log(titles(JSON.parse('{"title":"invoice"}')));
