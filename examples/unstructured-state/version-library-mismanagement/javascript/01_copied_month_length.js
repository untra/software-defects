function billingFebruary(year) {
    if (year % 4 === 0) {
        return 29;
    }
    return 28;
}
function payrollFebruary(_year) {
    return 28;
}
const year = 2024;
console.log(`billing ${year} february ${billingFebruary(year)}`);
console.log(`payroll ${year} february ${payrollFebruary(year)}`);
