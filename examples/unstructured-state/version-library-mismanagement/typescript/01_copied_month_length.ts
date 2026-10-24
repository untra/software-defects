function billingFebruary(year: number): number {
  if (year % 4 === 0) {
    return 29;
  }
  return 28;
}

function payrollFebruary(_year: number): number {
  return 28;
}

const year = 2024;
console.log(`billing ${year} february ${billingFebruary(year)}`);
console.log(`payroll ${year} february ${payrollFebruary(year)}`);
