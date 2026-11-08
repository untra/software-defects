type Release = { major: number; minor: number; patch: number; leapFebruary: boolean };

function acceptsCaret(found: Release): boolean {
  return found.major === 1 && found.minor >= 2;
}

const published: Release[] = [
  { major: 1, minor: 2, patch: 0, leapFebruary: true },
  { major: 1, minor: 9, patch: 0, leapFebruary: false },
];

let chosen = published[0];
for (const release of published) {
  if (acceptsCaret(release) && release.minor > chosen.minor) {
    chosen = release;
  }
}

const year = 2024;
const days = chosen.leapFebruary && year % 4 === 0 ? 29 : 28;
console.log(`february ${year}: ${days} via ${chosen.major}.${chosen.minor}.${chosen.patch}`);
