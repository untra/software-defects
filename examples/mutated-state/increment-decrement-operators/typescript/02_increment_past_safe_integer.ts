const ID_BATCH = 4;

let eventSequence = Number.MAX_SAFE_INTEGER - 1;
const issued = new Set<number>();
for (let i = 0; i < ID_BATCH; i++) issued.add(++eventSequence);
console.log(`issued ${ID_BATCH} ids, unique: ${issued.size}`);
