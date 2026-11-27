const CHUNK_BYTES = 4;
const records: string[] = [];

async function* readShipment(bytes: Buffer): AsyncGenerator<Buffer> {
  for (let offset = 0; offset < bytes.length; offset += CHUNK_BYTES) {
    yield bytes.subarray(offset, offset + CHUNK_BYTES);
  }
}

async function receiveShipment(names: string[]): Promise<void> {
  const bytes = Buffer.from(names.join(""));
  for await (const chunk of readShipment(bytes)) {
    records.push(chunk.toString());
  }
}

async function main(): Promise<void> {
  const shipment = ["pears", "figs-large"];
  await receiveShipment(shipment);
  console.log("records", JSON.stringify(records));
}

void main();
