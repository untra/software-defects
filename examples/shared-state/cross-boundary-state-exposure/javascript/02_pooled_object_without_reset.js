class FormState {
    fields = {};
}
const pool = [];
function acquireForm() {
    return pool.pop() ?? new FormState();
}
function releaseForm(form) {
    pool.push(form);
}
const aliceForm = acquireForm();
aliceForm.fields.ssn = "123-45-6789";
releaseForm(aliceForm);
const bobForm = acquireForm();
bobForm.fields.name = "bob";
console.log("bob's form:", bobForm.fields);
