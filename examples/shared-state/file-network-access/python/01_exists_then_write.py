from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import tempfile
import threading

NODES = ("ada", "ben")


def main():
    checked = threading.Barrier(len(NODES))

    def claim(node):
        if not path.exists():
            checked.wait()
            path.write_text(node, encoding="utf-8")
            return True
        return False

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "leader.lock"
        with ThreadPoolExecutor(max_workers=len(NODES)) as pool:
            leaders = list(pool.map(claim, NODES))
        print("leaders:", sum(leaders))


main()
