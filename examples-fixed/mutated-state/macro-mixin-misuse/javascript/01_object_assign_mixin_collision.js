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
const signup = { name: "ana", email: "", validate() {
        return RequiresEmail.validate.call(this) && RequiresName.validate.call(this);
    } };
console.log("signup valid:", signup.validate());
