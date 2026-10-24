import asyncio

INITIAL_STATUS = "packing"
NEXT_STATUS = "shipped"
primary = INITIAL_STATUS
replica = INITIAL_STATUS

async def copy(ready):
    global replica
    await ready.wait()
    replica = primary

async def main():
    global primary
    delivery = asyncio.Event()
    primary = NEXT_STATUS
    pending = asyncio.create_task(copy(delivery))
    print("status", replica)
    delivery.set()
    await pending

asyncio.run(main())
