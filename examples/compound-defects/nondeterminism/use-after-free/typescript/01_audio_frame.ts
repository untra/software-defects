const FRAME_BYTES = 8;
const FIRST_SAMPLE = 0;

function main(): void {
  const frame = new ArrayBuffer(FRAME_BYTES);
  const playback = frame;
  structuredClone(frame, { transfer: [frame] });
  console.log("playback detached", playback.detached);
  console.log("sample", new DataView(playback).getUint8(FIRST_SAMPLE));
}

main();
