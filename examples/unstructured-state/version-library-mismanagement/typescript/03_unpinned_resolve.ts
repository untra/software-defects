type Release = { version: string; leapFebruary: boolean };

const published: Release[] = [
  { version: "1.2.0", leapFebruary: true },
  { version: "1.2.1", leapFebruary: true },
  { version: "1.4.7", leapFebruary: false },
];

function resolveLatest(releases: Release[]): Release {
  return releases[releases.length - 1];
}

const chosen = resolveLatest(published);
const year = 2024;
const days = chosen.leapFebruary && year % 4 === 0 ? 29 : 28;
console.log(`february ${year}: ${days} via ${chosen.version}`);
