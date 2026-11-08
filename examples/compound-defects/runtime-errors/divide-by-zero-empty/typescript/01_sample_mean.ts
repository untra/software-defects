const SAMPLES: number[] = [];

function mean(samples: number[]): number {
  let total = 0;
  for (const sample of samples) {
    total += sample;
  }
  return total / samples.length;
}

function main(): void {
  console.log("mean", mean(SAMPLES));
}

main();
