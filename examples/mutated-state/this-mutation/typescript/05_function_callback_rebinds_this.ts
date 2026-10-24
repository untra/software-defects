const TTL_MS = 10;

class Session {
  expired = false;
  scheduleExpiry() {
    setTimeout(function () {
      this.expired = true;
    }, TTL_MS);
  }
}

const session = new Session();
session.scheduleExpiry();
setTimeout(() => console.log("expired:", session.expired), TTL_MS * 2);
