function birthday(user) {
    const nextAge = user.age + 1;
    return nextAge;
}
console.log("age", birthday({ age: 30 }));
