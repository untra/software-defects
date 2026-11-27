function normalize(user) {
    return { email: user.email.toLowerCase(), roles: [...user.roles, "member"] };
}
const auditSnapshot = { email: "Ana@Example.com", roles: ["admin"] };
const normalized = normalize(auditSnapshot);
console.log("normalized:", normalized.roles);
console.log("audit snapshot:", auditSnapshot.email, auditSnapshot.roles);
