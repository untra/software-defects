const GUESTS = ["Ari", "Bo", "Cy", "Dee"];

function roster(names: string[]): number {
  const accepted: string[] = [];
  let comparisons = 0;
  for (const name of names) {
    let present = false;
    for (const earlier of accepted) {
      comparisons++;
      if (earlier === name) present = true;
    }
    if (!present) accepted.push(name);
  }
  return comparisons;
}

function main(): void {
  console.log("comparisons", roster(GUESTS));
}

main();
