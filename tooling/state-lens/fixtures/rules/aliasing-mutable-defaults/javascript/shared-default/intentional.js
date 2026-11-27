const CACHE = {};
function lookup(key, store = CACHE) {
  if (key in store) {
    return store[key];
  }
  store[key] = key.length;
  return store[key];
}
lookup("a");
console.log(lookup("a"));
