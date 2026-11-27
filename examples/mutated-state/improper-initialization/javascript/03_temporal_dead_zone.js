function buildGreeting(name) {
    return `${GREETING_PREFIX}, ${name}`;
}
const GUEST_BANNER = buildGreeting("guest");
const GREETING_PREFIX = "Welcome";
console.log(GUEST_BANNER);
