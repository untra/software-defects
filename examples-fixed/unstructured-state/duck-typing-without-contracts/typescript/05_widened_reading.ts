const FAHRENHEIT_SCALE = 1.8;
const FAHRENHEIT_OFFSET = 32;

const KELVIN_OFFSET = 273.15;
type Reading = { value: number; unit: "C" | "K" };

function fahrenheit(reading: Reading): number {
  return (reading.unit === "K" ? reading.value - KELVIN_OFFSET : reading.value) * FAHRENHEIT_SCALE + FAHRENHEIT_OFFSET;
}

const room: Reading = { value: 100, unit: "C" };
const lab: Reading = { value: 100, unit: "K" };
console.log("room", fahrenheit(room));
console.log("lab", fahrenheit(lab));
