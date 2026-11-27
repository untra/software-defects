const BYTES_PER_KB = 1024;

const archiveBytes = 3_000_000_000_000;
const archiveKB = Math.floor(archiveBytes / BYTES_PER_KB);
console.log("archive size KB:", archiveKB);
