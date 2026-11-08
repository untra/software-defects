const db = new Map([["ana@example.com", "Ana"]]);
const cache = new Map();
const normalize = (email) => email.trim().toLowerCase();
function displayName(email) {
    const key = normalize(email);
    const hit = cache.get(key);
    if (hit !== undefined)
        return hit;
    const name = db.get(key) ?? "unknown";
    cache.set(key, name);
    return name;
}
function rename(email, name) {
    db.set(normalize(email), name);
    cache.delete(normalize(email));
}
displayName("Ana@Example.com");
rename("Ana@Example.com", "Ana Lopez");
console.log("greeting:", displayName("ana@example.com"));
