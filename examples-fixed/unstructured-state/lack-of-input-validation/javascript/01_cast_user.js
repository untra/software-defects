const body = "{}";
const user = JSON.parse(body);
if (typeof user === "object" && user !== null && "email" in user && typeof user.email === "string") {
    console.log(user.email.toLowerCase());
}
else {
    console.log("body rejected");
}
