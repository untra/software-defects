import asyncio


async def save():
    raise RuntimeError("disk full")


async def run():
    try:
        await save()
    except RuntimeError as error:
        print(f"save failed: {error}")


def main():
    asyncio.run(run())


main()
