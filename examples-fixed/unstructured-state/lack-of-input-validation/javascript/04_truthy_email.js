function acceptEmail(email) {
    if (!email.trim()) {
        return "missing";
    }
    return "accepted " + email;
}
console.log(acceptEmail(" "));
