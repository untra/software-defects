function februaryDays(year: number, leap: boolean): number {
  if (leap && year % 4 === 0) {
    return 29;
  }
  return 28;
}

const year = 2024;
console.log("dates-v1", februaryDays(year, true));
console.log("dates-v2", februaryDays(year, false));
