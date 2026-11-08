type User = { email: string; roles: string[] };

function normalize(user: User): User {
  user.email = user.email.toLowerCase();
  user.roles.push("member");
  return user;
}

const auditSnapshot: User = { email: "Ana@Example.com", roles: ["admin"] };
const normalized = normalize(auditSnapshot);
console.log("normalized:", normalized.roles);
console.log("audit snapshot:", auditSnapshot.email, auditSnapshot.roles);
