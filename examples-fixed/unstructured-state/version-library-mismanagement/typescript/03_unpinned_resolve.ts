type Release = { version: string; leapFebruary: boolean };

const published: Release[] = [
  { version: "1.2.0", leapFebruary: true },
  { version: "1.2.1", leapFebruary: true },
  { version: "1.4.7", leapFebruary: false },
];

function resolvePinned(releases: Release[]): Release {
  const pinned = releases.find(release => release.version === "1.2.1");
  if (!pinned) throw new Error("pinned version unavailable");
  return pinned;
}

const chosen = resolvePinned(published);
const year = 2024;
const days = chosen.leapFebruary && year % 4 === 0 ? 29 : 28;
console.log(`february ${year}: ${days} via ${chosen.version}`);
