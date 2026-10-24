import asyncio
from pathlib import Path
from tempfile import TemporaryDirectory

DIRECTORY = "reports"

async def initialize(path, ready):
    if not path.exists():
        await ready.wait()
        path.mkdir()

async def main():
    with TemporaryDirectory(prefix="reports-") as directory:
        path = Path(directory) / DIRECTORY
        ready = asyncio.Event()
        pending = asyncio.create_task(initialize(path, ready))
        await asyncio.sleep(0)
        path.mkdir()
        ready.set()
        try:
            await pending
        except FileExistsError:
            print("directory already occupied")

asyncio.run(main())
