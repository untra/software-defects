function requireAdmin(users) {
    for (const user of users) {
        if (user.role === "admin") return user;
    }
    throw new Error("administrator required");
}
try {
    console.log("admin", requireAdmin([{ name: "pam", role: "guest" }]));
} catch (error) {
    console.log(error instanceof Error ? error.message : "lookup failed");
}
