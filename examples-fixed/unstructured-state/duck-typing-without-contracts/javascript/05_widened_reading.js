const FAHRENHEIT_SCALE = 1.8;
const FAHRENHEIT_OFFSET = 32;
const KELVIN_OFFSET = 273.15;
function fahrenheit(reading) {
    if (!reading || !Number.isFinite(reading.value) || !["C", "K"].includes(reading.unit)) throw new TypeError("Expected a finite C or K reading");
    return (reading.unit === "K" ? reading.value - KELVIN_OFFSET : reading.value) * FAHRENHEIT_SCALE + FAHRENHEIT_OFFSET;
}
const room = { value: 100, unit: "C" };
const lab = { value: 100, unit: "K" };
console.log("room", fahrenheit(room));
console.log("lab", fahrenheit(lab));
