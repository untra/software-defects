const routes = {};
const ROUTE = "shipping";
const STANDARD = "standard";
const EXPRESS = "express";
function gate() {
    let open = () => undefined;
    const promise = new Promise((resolve) => { open = resolve; });
    return { promise, open };
}
async function install(module, ready) {
    await ready;
    Object.assign(routes, module);
}
async function main() {
    const standardReady = gate();
    const expressReady = gate();
    const standard = install({ [ROUTE]: () => STANDARD }, standardReady.promise);
    const express = install({ [ROUTE]: () => EXPRESS }, expressReady.promise);
    expressReady.open();
    await express;
    standardReady.open();
    await standard;
    console.log("shipping", routes[ROUTE]());
}
void main();
