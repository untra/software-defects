function birthday(user: { age?: number }): number | null {
  return user.age === undefined ? null : user.age + 1;
}

console.log("age", birthday({}));
