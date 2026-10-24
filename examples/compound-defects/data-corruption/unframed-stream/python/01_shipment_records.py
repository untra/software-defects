import asyncio

CHUNK_BYTES = 4
SHIPMENT = ["pears", "figs-large"]
records = []

async def read_shipment(data):
    for offset in range(0, len(data), CHUNK_BYTES):
        await asyncio.sleep(0)
        yield data[offset:offset + CHUNK_BYTES]

async def receive(names):
    data = "".join(names).encode()
    async for chunk in read_shipment(data):
        records.append(chunk.decode())

async def main():
    await receive(SHIPMENT)
    print("records", records)

asyncio.run(main())
