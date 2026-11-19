import asyncio

PEERS = ("billing", "shipping")


async def run():
    started = set()
    released = asyncio.Event()

    async def register(peer):
        started.add(peer)
        await released.wait()

    async def coordinate():
        for peer in PEERS:
            await register(peer)

    work = asyncio.create_task(coordinate())
    for _ in range(len(PEERS) + 1):
        await asyncio.sleep(0)
    print("all peers registered:", len(started) == len(PEERS))
    released.set()
    await work


asyncio.run(run())
