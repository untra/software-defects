import { EventEmitter } from "node:events";

const REQUESTS = 100;

const configBus = new EventEmitter();
configBus.setMaxListeners(0);

function handleRequest(id: number): string {
  const onReload = () => console.log(`request ${id} saw reload`);
  configBus.on("reload", onReload);
  try { return `response ${id}`; }
  finally { configBus.off("reload", onReload); }
}

for (let i = 0; i < REQUESTS; i++) handleRequest(i);
console.log("reload listeners after requests finished:", configBus.listenerCount("reload"));
