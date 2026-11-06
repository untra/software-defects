const RequiresEmail = {
    validate() {
        return Boolean(this.email);
    },
};
const RequiresName = {
    validate() {
        return Boolean(this.name);
    },
};
const signup = Object.assign({ name: "ana", email: "" }, RequiresEmail, RequiresName);
console.log("signup valid:", signup.validate());
