const raw = '{"name":"ada","admin":"true"}';
const user = JSON.parse(raw, (_key, value) => {
  if (_key === "admin") return undefined;
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  return value;
});
console.log("admin", user.admin);
