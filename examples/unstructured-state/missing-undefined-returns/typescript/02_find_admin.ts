function requireAdmin(users: { name: string; role: string }[]): { name: string } {
    for (const user of users) {
        if (user.role === "admin") return user;
    }
}
try {
    console.log("admin", requireAdmin([{ name: "pam", role: "guest" }]));
} catch (error) {
    console.log(error instanceof Error ? error.message : "lookup failed");
}
