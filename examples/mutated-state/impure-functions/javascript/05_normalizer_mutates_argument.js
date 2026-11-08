function normalize(user) {
    user.email = user.email.toLowerCase();
    user.roles.push("member");
    return user;
}
const auditSnapshot = { email: "Ana@Example.com", roles: ["admin"] };
const normalized = normalize(auditSnapshot);
console.log("normalized:", normalized.roles);
console.log("audit snapshot:", auditSnapshot.email, auditSnapshot.roles);
