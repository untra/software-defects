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

const signup = Object.assign({ name: "ana", email: "" }, RequiresEmail, RequiresName);
console.log("signup valid:", signup.validate());
