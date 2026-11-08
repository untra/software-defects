type Routes = Record<string, () => string>;
const routes: Routes = {};
const ROUTE = "shipping";
const STANDARD = "standard";
const EXPRESS = "express";

function gate(): { promise: Promise<void>; open: () => void } {
  let open: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => { open = resolve; });
  return { promise, open };
}

async function install(module: Routes, ready: Promise<void>): Promise<void> {
  await ready;
  Object.assign(routes, module);
}

async function main(): Promise<void> {
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
