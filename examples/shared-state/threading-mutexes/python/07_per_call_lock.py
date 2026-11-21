import asyncio


async def main():
    count = 0

    async def record():
        nonlocal count
        lock = asyncio.Lock()
        async with lock:
            saved = count
            await asyncio.sleep(0)
            count = saved + 1

    await asyncio.gather(record(), record())
    print("count:", count)


asyncio.run(main())
