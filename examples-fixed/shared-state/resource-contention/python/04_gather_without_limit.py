import asyncio


async def work():
    await asyncio.sleep(0)
    return 1


async def run():
    done = []
    for _ in range(8):
        done.append(await work())
    print(len(done))


def main():
    asyncio.run(run())


main()
