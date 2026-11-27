const ARRIVALS_PER_TICK = 1000;
const HANDLED_PER_TICK = 10;
const TICKS = 10;

const MAX_QUEUE = 100;
const queue: string[] = [];

function enqueue(event: string): boolean {
  if (queue.length >= MAX_QUEUE) return false;
  queue.push(event);
  return true;
}

function drain(): void {
  queue.splice(0, HANDLED_PER_TICK);
}

for (let tick = 0; tick < TICKS; tick++) {
  for (let i = 0; i < ARRIVALS_PER_TICK; i++) enqueue(`event-${tick}-${i}`);
  drain();
}
console.log("queued events in memory:", queue.length);
