let currentLocale = "en-US";
const cache = new Map<string, string>();

function formatPrice(amount: number): string {
  const key = JSON.stringify([currentLocale, amount]);
  const hit = cache.get(key);
  if (hit !== undefined) return hit;
  const formatted = new Intl.NumberFormat(currentLocale, { style: "currency", currency: "EUR" }).format(amount);
  cache.set(key, formatted);
  return formatted;
}

console.log(formatPrice(1234.5));
currentLocale = "de-DE";
console.log(formatPrice(1234.5));
