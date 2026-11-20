import asyncio


async def quote(kind):
    if kind == "member":
        return 9
    return 10


def main():
    print(asyncio.run(quote("guest")))


main()
