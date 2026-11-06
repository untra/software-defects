const FAHRENHEIT_SCALE = 1.8;
const FAHRENHEIT_OFFSET = 32;
function fahrenheit(reading) {
    return reading.value * FAHRENHEIT_SCALE + FAHRENHEIT_OFFSET;
}
const room = { value: 100, unit: "C" };
const lab = { value: 100, unit: "K" };
console.log("room", fahrenheit(room));
console.log("lab", fahrenheit(lab));
