const ROUNDS = 20;
function deliver(job) {
    throw new Error(`downstream timeout for job ${job.id}`);
}
function runRound(queue) {
    const retries = [];
    for (const job of queue) {
        try {
            deliver(job);
        }
        catch {
            retries.push(job, { ...job, attempt: job.attempt + 1 });
        }
    }
    return retries;
}
let queue = [{ id: 1, attempt: 0 }];
for (let round = 0; round < ROUNDS; round++)
    queue = runRound(queue);
console.log("jobs queued after", ROUNDS, "rounds:", queue.length);
