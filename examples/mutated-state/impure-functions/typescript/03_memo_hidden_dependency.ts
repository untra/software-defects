let currentLocale = "en-US";
const cache = new Map<number, string>();

function formatPrice(amount: number): string {
  const hit = cache.get(amount);
  if (hit !== undefined) return hit;
  const formatted = new Intl.NumberFormat(currentLocale, { style: "currency", currency: "EUR" }).format(amount);
  cache.set(amount, formatted);
  return formatted;
}

console.log(formatPrice(1234.5));
currentLocale = "de-DE";
console.log(formatPrice(1234.5));
