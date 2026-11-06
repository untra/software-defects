class ReportApi {
    token = "valid";
    fetch() {
        return "quarterly revenue report";
    }
}
const Authorized = (Base) => class extends Base {
    fetch() {
        return this.token ? super.fetch() : "403 denied";
    }
};
const Cached = (Base) => class extends Base {
    cached = null;
    fetch() {
        this.cached ??= super.fetch();
        return this.cached;
    }
};
const SecureApi = Authorized(Cached(ReportApi));
const api = new SecureApi();
console.log("before revoke:", api.fetch());
api.token = null;
console.log("after revoke: ", api.fetch());
