function billingFebruary(year: number): number {
  if (year % 400 === 0 || (year % 4 === 0 && year % 100 !== 0)) {
    return 29;
  }
  return 28;
}

function payrollFebruary(year: number): number {
  return billingFebruary(year);
}

const year = 2024;
console.log(`billing ${year} february ${billingFebruary(year)}`);
console.log(`payroll ${year} february ${payrollFebruary(year)}`);
