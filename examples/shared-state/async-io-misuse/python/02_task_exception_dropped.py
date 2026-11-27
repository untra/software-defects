import asyncio


async def save():
    raise RuntimeError("disk full")


async def run():
    asyncio.create_task(save())
    await asyncio.sleep(0)
    print("done")


def main():
    asyncio.run(run())


main()
