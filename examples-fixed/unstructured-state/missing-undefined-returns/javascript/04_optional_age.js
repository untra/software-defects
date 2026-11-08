function birthday(user) {
    return user.age === undefined ? null : user.age + 1;
}
console.log("age", birthday({}));
