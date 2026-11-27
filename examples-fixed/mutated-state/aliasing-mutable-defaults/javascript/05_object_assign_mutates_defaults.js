const DEFAULT_SETTINGS = { theme: "light", fontSize: 14 };
function resolveSettings(overrides) {
    return Object.assign({}, DEFAULT_SETTINGS, overrides);
}
const alice = resolveSettings({ theme: "dark" });
const bob = resolveSettings({ fontSize: 18 });
console.log("alice", alice);
console.log("bob", bob);
