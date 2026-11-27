const RequiresEmail = {
  validate(this: { email?: string }) {
    return Boolean(this.email);
  },
};

const RequiresName = {
  validate(this: { name?: string }) {
    return Boolean(this.name);
  },
};

const signup = { name: "ana", email: "", validate() {
  return RequiresEmail.validate.call(this) && RequiresName.validate.call(this);
} };
console.log("signup valid:", signup.validate());
