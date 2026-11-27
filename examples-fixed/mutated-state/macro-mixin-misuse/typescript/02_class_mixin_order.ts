type Ctor<T> = new (...args: any[]) => T;

class ReportApi {
  token: string | null = "valid";
  fetch(): string {
    return "quarterly revenue report";
  }
}

const Authorized = <B extends Ctor<ReportApi>>(Base: B) =>
  class extends Base {
    fetch() {
      return this.token ? super.fetch() : "403 denied";
    }
  };

const Cached = <B extends Ctor<ReportApi>>(Base: B) =>
  class extends Base {
    cached: string | null = null;
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
