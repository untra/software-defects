import asyncio


async def run():
    winner_done = asyncio.Event()
    loser_go = asyncio.Event()
    page = ""

    async def fast():
        nonlocal page
        page = "winner"
        winner_done.set()

    async def slow():
        nonlocal page
        await loser_go.wait()
        page = "later"

    first = asyncio.create_task(fast())
    second = asyncio.create_task(slow())
    await winner_done.wait()
    print(page)
    loser_go.set()
    await second
    await first
    print(page)


def main():
    asyncio.run(run())


main()
