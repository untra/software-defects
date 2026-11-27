import asyncio


async def save():
    return "saved"


def main():
    print(asyncio.run(save()))
    print("done")


main()
