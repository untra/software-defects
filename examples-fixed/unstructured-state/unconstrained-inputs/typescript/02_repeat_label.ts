const MAX_LABEL_LENGTH = 6;
const requested = 8;
const label = Number.isInteger(requested) && requested >= 0 && requested <= MAX_LABEL_LENGTH ? "x".repeat(requested) : "rejected";
console.log("label", label);
