import asyncio


async def quote(kind):
    if kind == "member":
        return 9


def main():
    print(asyncio.run(quote("guest")))


main()
