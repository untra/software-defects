const ALLOWED_PREFIX = "https://safe.example";
function allowed(raw) {
    return raw.startsWith(ALLOWED_PREFIX);
}
console.log("allowed", allowed("https://safe.example.other.test/a"));
