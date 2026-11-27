function buildGreeting(name: string): string {
  return `${GREETING_PREFIX}, ${name}`;
}

const GREETING_PREFIX = "Welcome";
const GUEST_BANNER = buildGreeting("guest");

console.log(GUEST_BANNER);
