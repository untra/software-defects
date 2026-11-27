import asyncio

MILLIS = 1700000000000
PREFIX = "receipt-"
clock = {"millis": MILLIS}
receipts = {}

async def issue(customer, ready):
    identifier = PREFIX + str(clock["millis"])
    available = identifier not in receipts
    await ready.wait()
    if available:
        receipts[identifier] = customer
    return identifier

async def main():
    ready = asyncio.Event()
    first = asyncio.create_task(issue("orchard", ready))
    second = asyncio.create_task(issue("harbor", ready))
    await asyncio.sleep(0)
    ready.set()
    identifiers = await asyncio.gather(first, second)
    print("distinct ids", len(set(identifiers)))
    print("receipts", len(receipts))

asyncio.run(main())
