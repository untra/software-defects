const MAX_BUFFER_BYTES = 8;

function allocate(requested: number) {
  return new Uint8Array(requested);
}

try {
  console.log("bytes", allocate(MAX_BUFFER_BYTES + 1).length);
} catch (error) {
  console.log(error instanceof Error ? error.message : "buffer rejected");
}
