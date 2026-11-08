const ISO_DATE_LENGTH = 10;
const JANUARY = 0;
const LAST_DAY_OF_JANUARY = 31;

function nextBillingDate(from: Date): Date {
  const next = new Date(from);
  const day = next.getUTCDate();
  next.setUTCDate(1);
  next.setUTCMonth(next.getUTCMonth() + 1);
  const lastDay = new Date(Date.UTC(next.getUTCFullYear(), next.getUTCMonth() + 1, 0)).getUTCDate();
  next.setUTCDate(Math.min(day, lastDay));
  return next;
}

const signup = new Date(Date.UTC(2026, JANUARY, LAST_DAY_OF_JANUARY));
console.log("next billing:", nextBillingDate(signup).toISOString().slice(0, ISO_DATE_LENGTH));
