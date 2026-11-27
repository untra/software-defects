import asyncio

ROUTE = "shipping"
STANDARD = "standard"
EXPRESS = "express"
routes = {}

async def install(module, ready):
    await ready.wait()
    routes.update(module)

async def main():
    standard_ready = asyncio.Event()
    express_ready = asyncio.Event()
    first = asyncio.create_task(install({ROUTE: lambda: STANDARD}, standard_ready))
    second = asyncio.create_task(install({ROUTE: lambda: EXPRESS}, express_ready))
    express_ready.set()
    await second
    standard_ready.set()
    await first
    print("shipping", routes[ROUTE]())

asyncio.run(main())
