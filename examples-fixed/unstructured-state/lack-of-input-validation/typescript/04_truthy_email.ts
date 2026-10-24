function acceptEmail(email: string): string {
  if (!email.trim()) {
    return "missing";
  }
  return "accepted " + email;
}

console.log(acceptEmail(" "));
