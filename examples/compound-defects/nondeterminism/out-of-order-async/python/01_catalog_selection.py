import asyncio

shown = ""

async def load(name, ready):
    global shown
    await ready.wait()
    shown = name

async def main():
    earlier = asyncio.Event()
    later = asyncio.Event()
    first = asyncio.create_task(load("orchard", earlier))
    second = asyncio.create_task(load("harbor", later))
    later.set()
    await second
    earlier.set()
    await first
    print("shown", shown)

asyncio.run(main())
