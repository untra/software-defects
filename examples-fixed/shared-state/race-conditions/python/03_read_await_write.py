import asyncio


async def sell_both():
    stock = 1
    sold = []

    async def sell(name):
        nonlocal stock
        if stock > 0:
            stock -= 1
            try:
                await asyncio.sleep(0)
                sold.append(name)
            except BaseException:
                stock += 1
                raise

    await asyncio.gather(sell("ada"), sell("ben"))
    print(len(sold), stock)


def main():
    asyncio.run(sell_both())


main()
