import asyncio
import threading


async def main():
    lock = threading.RLock()
    values = []
    count = 0

    async def record():
        nonlocal count
        with lock:
            saved = count
            await asyncio.sleep(0)
            count = saved + 1
            values.append(count)

    await asyncio.gather(record(), record())
    print("count:", count)


asyncio.run(main())
