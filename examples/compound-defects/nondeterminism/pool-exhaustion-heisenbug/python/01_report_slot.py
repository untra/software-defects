import asyncio

CAPACITY = 1
START_TICK = 100
FINISH_TICK = 105
occupied = 0
tick = START_TICK

async def report(ready):
    global occupied
    if occupied == CAPACITY:
        print("report pool busy at", tick)
        return
    occupied += 1
    started = tick
    try:
        await ready.wait()
        print("held ticks", tick - started)
    finally:
        occupied -= 1

async def main():
    global tick
    delivery = asyncio.Event()
    first = asyncio.create_task(report(delivery))
    await asyncio.sleep(0)
    tick = FINISH_TICK
    await report(delivery)
    delivery.set()
    await first

asyncio.run(main())
