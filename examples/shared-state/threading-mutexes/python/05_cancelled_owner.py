import asyncio


async def main():
    lock = asyncio.Lock()
    entered = asyncio.Event()

    async def save():
        await lock.acquire()
        entered.set()
        await asyncio.Event().wait()
        lock.release()

    task = asyncio.create_task(save())
    await entered.wait()
    task.cancel()
    try:
        await task
    except asyncio.CancelledError:
        pass
    print("locked:", lock.locked())


asyncio.run(main())
