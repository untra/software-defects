import asyncio


async def main():
    lock = asyncio.Lock()
    entered = asyncio.Event()

    async def save():
        async with lock:
            entered.set()
            await asyncio.Event().wait()

    task = asyncio.create_task(save())
    await entered.wait()
    task.cancel()
    try:
        await task
    except asyncio.CancelledError:
        pass
    print("locked:", lock.locked())


asyncio.run(main())
