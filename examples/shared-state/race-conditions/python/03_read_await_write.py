import asyncio


async def sell_both():
    stock = 1
    sold = []

    async def sell(name):
        nonlocal stock
        seen = stock
        await asyncio.sleep(0)
        if seen > 0:
            stock = seen - 1
            sold.append(name)

    await asyncio.gather(sell("ada"), sell("ben"))
    print(len(sold), stock)


def main():
    asyncio.run(sell_both())


main()
