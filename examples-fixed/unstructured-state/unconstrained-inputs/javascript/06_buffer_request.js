const MAX_BUFFER_BYTES = 8;

function allocate(requested) {
  if (!Number.isSafeInteger(requested) || requested < 0 || requested > MAX_BUFFER_BYTES) {
    throw new Error("buffer rejected");
  }
  return new Uint8Array(requested);
}

try {
  console.log("bytes", allocate(MAX_BUFFER_BYTES + 1).length);
} catch (error) {
  console.log(error.message);
}
