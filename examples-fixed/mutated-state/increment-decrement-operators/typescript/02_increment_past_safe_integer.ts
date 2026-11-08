const ID_BATCH = 4;

let eventSequence = BigInt(Number.MAX_SAFE_INTEGER) - 1n;
const issued = new Set<bigint>();
for (let i = 0; i < ID_BATCH; i++) issued.add(++eventSequence);
console.log(`issued ${ID_BATCH} ids, unique: ${issued.size}`);
