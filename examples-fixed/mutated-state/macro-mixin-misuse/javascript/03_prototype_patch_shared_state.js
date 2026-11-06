class HttpClient {
    send(url) {
        return `GET ${url}`;
    }
}
function trackCalls(cls) {
    const send = cls.prototype.send;
    cls.prototype.send = function (url) {
        if (!Object.hasOwn(this, "calls"))
            this.calls = [];
        this.calls.push(url);
        return send.call(this, url);
    };
}
trackCalls(HttpClient);
const billing = new HttpClient();
const search = new HttpClient();
billing.send("/invoices");
search.send("/search?q=boots");
console.log("billing calls:", billing.calls);
