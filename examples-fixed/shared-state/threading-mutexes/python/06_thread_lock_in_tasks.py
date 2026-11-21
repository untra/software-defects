import asyncio


async def main():
    lock = asyncio.Lock()
    values = []
    count = 0

    async def record():
        nonlocal count
        async with lock:
            saved = count
            await asyncio.sleep(0)
            count = saved + 1
            values.append(count)

    await asyncio.gather(record(), record())
    print("count:", count)


asyncio.run(main())
