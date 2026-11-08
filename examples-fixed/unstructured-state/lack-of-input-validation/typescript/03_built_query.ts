const name = "O'Brien";
const query = "SELECT id FROM users WHERE name = ?";
const parameters = [name];
console.log(query, parameters);
