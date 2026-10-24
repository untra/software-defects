const MAX_ATTEMPTS = 3;
const ROUNDS = 20;

type Job = { id: number; attempt: number };

function deliver(job: Job): void {
  throw new Error(`downstream timeout for job ${job.id}`);
}

function runRound(queue: Job[]): Job[] {
  const retries: Job[] = [];
  for (const job of queue) {
    try {
      deliver(job);
    } catch {
      if (job.attempt + 1 < MAX_ATTEMPTS) retries.push({ ...job, attempt: job.attempt + 1 });
    }
  }
  return retries;
}

let queue: Job[] = [{ id: 1, attempt: 0 }];
for (let round = 0; round < ROUNDS; round++) queue = runRound(queue);
console.log("jobs queued after", ROUNDS, "rounds:", queue.length);
