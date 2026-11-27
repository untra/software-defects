function acceptEmail(email: unknown) {
  if (typeof email !== "string" || !email.trim()) {
    return "missing";
  }
  return "accepted " + email;
}

console.log(acceptEmail(" "));
