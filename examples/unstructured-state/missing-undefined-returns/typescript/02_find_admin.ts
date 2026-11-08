function findAdmin(users: { name: string; role: string }[]): { name: string } {
  for (const user of users) {
    if (user.role === "admin") {
      return user;
    }
  }
}

const admin = findAdmin([{ name: "pam", role: "guest" }]);
console.log(admin.name);
