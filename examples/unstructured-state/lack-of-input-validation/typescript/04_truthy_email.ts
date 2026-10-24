function acceptEmail(email: string): string {
  if (!email) {
    return "missing";
  }
  return "accepted " + email;
}

console.log(acceptEmail(" "));
