const TOP_N = 3;

function topReadings(readings: number[]): number[] {
  return [...readings].sort((a, b) => b - a).slice(0, TOP_N);
}

const readingsByTime = [12, 48, 7, 33, 21];
console.log("top:", topReadings(readingsByTime));
console.log("latest reading:", readingsByTime[readingsByTime.length - 1]);
