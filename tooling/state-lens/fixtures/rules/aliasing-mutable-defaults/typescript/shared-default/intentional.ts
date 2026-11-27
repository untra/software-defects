const CACHE: Record<string, number> = {};
function lookup(key: string, store: Record<string, number> = CACHE) {
  if (key in store) {
    return store[key];
  }
  store[key] = key.length;
  return store[key];
}
lookup("a");
console.log(lookup("a"));
