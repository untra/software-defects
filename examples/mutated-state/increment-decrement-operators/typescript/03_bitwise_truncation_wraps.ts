const BYTES_PER_KB = 1024;

const archiveBytes = 3_000_000_000_000;
const archiveKB = (archiveBytes / BYTES_PER_KB) | 0;
console.log("archive size KB:", archiveKB);
