function titles(payload: unknown): string[] {
    const records = payload as { title: string }[];
    return records.map((record) => record.title.toUpperCase());
}
console.log(titles(JSON.parse('{"title":"invoice"}')));
