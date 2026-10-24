class HttpClient {
  send(url: string) {
    return `GET ${url}`;
  }
}

function trackCalls(cls: { prototype: any }) {
  cls.prototype.calls = [];
  const send = cls.prototype.send;
  cls.prototype.send = function (this: any, url: string) {
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
