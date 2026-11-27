const row = JSON.parse('{"id":"1","constructor":{"name":"Array"}}', (key, value) => key === "constructor" || key === "__proto__" ? undefined : value);
console.log("kind", row.constructor.name);
