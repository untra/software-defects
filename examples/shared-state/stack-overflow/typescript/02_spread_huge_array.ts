const READINGS = 1_000_000;

const readings = Array.from({ length: READINGS }, (_, i) => i % 97);

try {
  console.log("peak reading:", Math.max(...readings));
} catch (e) {
  console.log(`${(e as Error).name}: ${(e as Error).message}`);
}
