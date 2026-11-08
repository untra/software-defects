import asyncio


async def work():
    await asyncio.sleep(0)
    return 1


async def run():
    tasks = [work() for _ in range(8)]
    done = await asyncio.gather(*tasks)
    print(len(done))


def main():
    asyncio.run(run())


main()
