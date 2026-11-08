class HttpClient {
  send(url: string) {
    return `GET ${url}`;
  }
}

function trackCalls(cls: { prototype: any }) {
  const send = cls.prototype.send;
  cls.prototype.send = function (this: any, url: string) {
    if (!Object.hasOwn(this, "calls")) this.calls = [];
    this.calls.push(url);
    return send.call(this, url);
  };
}

trackCalls(HttpClient);
const billing = new HttpClient() as any;
const search = new HttpClient() as any;
billing.send("/invoices");
search.send("/search?q=boots");
console.log("billing calls:", billing.calls);
