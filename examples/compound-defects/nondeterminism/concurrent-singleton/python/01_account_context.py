import asyncio

class AccountContext:
    def __init__(self):
        self.user = ""

    async def show(self, user, ready):
        self.user = user
        await ready.wait()
        return self.user

context = AccountContext()

async def main():
    first_ready = asyncio.Event()
    second_ready = asyncio.Event()
    first = asyncio.create_task(context.show("orchard", first_ready))
    await asyncio.sleep(0)
    second_ready.set()
    await context.show("harbor", second_ready)
    first_ready.set()
    print("orchard received", await first)

asyncio.run(main())
