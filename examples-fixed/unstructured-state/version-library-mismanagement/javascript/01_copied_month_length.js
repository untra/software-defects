function billingFebruary(year) {
    if (year % 400 === 0 || (year % 4 === 0 && year % 100 !== 0)) {
        return 29;
    }
    return 28;
}
function payrollFebruary(year) {
    return billingFebruary(year);
}
const year = 2024;
console.log(`billing ${year} february ${billingFebruary(year)}`);
console.log(`payroll ${year} february ${payrollFebruary(year)}`);
