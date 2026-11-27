type User = { email: string; roles: string[] };

function normalize(user: User): User {
  return { email: user.email.toLowerCase(), roles: [...user.roles, "member"] };
}

const auditSnapshot: User = { email: "Ana@Example.com", roles: ["admin"] };
const normalized = normalize(auditSnapshot);
console.log("normalized:", normalized.roles);
console.log("audit snapshot:", auditSnapshot.email, auditSnapshot.roles);
