function headersFor(token: string) {
  return { Authorization: token };
}
const first = headersFor("a");
const second = headersFor("b");
first["Authorization"] = "";
console.log(second);
