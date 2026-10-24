const person: { name: string; age?: number } = { name: "pam" };
console.log("next", person.age === undefined ? "unavailable" : person.age + 1);
