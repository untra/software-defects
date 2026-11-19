from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import tempfile
import threading

NODES = ("ada", "ben")


def main():
    checked = threading.Barrier(len(NODES))

    def claim(node):
        checked.wait()
        try:
            with path.open("x", encoding="utf-8") as handle:
                handle.write(node)
            return True
        except FileExistsError:
            return False

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "leader.lock"
        with ThreadPoolExecutor(max_workers=len(NODES)) as pool:
            leaders = list(pool.map(claim, NODES))
        print("leaders:", sum(leaders))


main()
