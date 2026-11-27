const PING_DELAY_MS = 10;
const endpoints = ["/health", "/ready", "/metrics"];

for (var i = 0; i < endpoints.length; i++) {
  setTimeout(() => console.log("pinging", endpoints[i]), PING_DELAY_MS);
}
