function birthday(user: { age?: number }): number {
  return user.age + 1;
}

console.log("age", birthday({}));
