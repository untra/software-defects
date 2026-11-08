import asyncio


async def step(name, log):
    log.append(name)
    await asyncio.sleep(0)
    log.append(name)


async def run():
    log = []
    for name in ("a", "b"):
        await step(name, log)
    print(",".join(log))


def main():
    asyncio.run(run())


main()
