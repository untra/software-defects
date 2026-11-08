const FRAME_BYTES = 1024;
const FILL = 7;

function sendToEncoder(buffer: ArrayBuffer): ArrayBuffer {
  return structuredClone(buffer, { transfer: [buffer] });
}

const frame = new Uint8Array(FRAME_BYTES).fill(FILL);
sendToEncoder(frame.buffer);
const checksum = frame.reduce((sum, b) => sum + b, 0);
console.log("frame bytes:", frame.length, "checksum:", checksum);
