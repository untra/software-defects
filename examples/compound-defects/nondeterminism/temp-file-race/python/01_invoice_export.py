import asyncio
from pathlib import Path
from tempfile import TemporaryDirectory

PREFIX = "invoice-"
STAMP = 1700000000000
DOCUMENT = "orchard invoice"
OTHER_DOCUMENT = "harbor invoice"

async def save(path, ready):
    if not path.exists():
        await ready.wait()
        path.write_text(DOCUMENT)

async def main():
    with TemporaryDirectory(prefix=PREFIX) as directory:
        path = Path(directory) / (PREFIX + str(STAMP))
        ready = asyncio.Event()
        pending = asyncio.create_task(save(path, ready))
        await asyncio.sleep(0)
        path.write_text(OTHER_DOCUMENT)
        ready.set()
        await pending
        print("existing document", path.read_text())

asyncio.run(main())
