const MAX_COPIES = 4;
const MAX_INPUT_ITEMS = 4;
const MAX_OUTPUT_ITEMS = 6;

function expand(items: string[], copies: number) {
  if (!Number.isSafeInteger(copies) || copies < 0 || copies > MAX_COPIES || items.length > MAX_INPUT_ITEMS) {
    throw new Error("input rejected");
  }
  if (copies !== 0 && items.length > Math.floor(MAX_OUTPUT_ITEMS / copies)) {
    throw new Error("expansion rejected");
  }
  const output: string[] = [];
  for (let copy = 0; copy < copies; copy++) {
    output.push(...items);
  }
  return output;
}

try {
  console.log("expanded", expand(["a", "b", "c"], 3).length);
} catch (error) {
  console.log(error instanceof Error ? error.message : "expansion rejected");
}
