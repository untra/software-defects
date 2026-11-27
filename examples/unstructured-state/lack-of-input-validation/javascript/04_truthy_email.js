function acceptEmail(email) {
    if (!email) {
        return "missing";
    }
    return "accepted " + email;
}
console.log(acceptEmail(" "));
