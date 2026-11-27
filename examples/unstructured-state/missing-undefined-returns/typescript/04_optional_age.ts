function birthday(user: { age: number }): number {
    const nextAge = user.age + 1;
    nextAge;
}
console.log("age", birthday({ age: 30 }));
