type User = { email: string };

const body = "{}";
const user = JSON.parse(body) as User;
console.log(user.email.toLowerCase());
