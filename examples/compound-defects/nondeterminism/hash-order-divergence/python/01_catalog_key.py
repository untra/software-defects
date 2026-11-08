import os
from pathlib import Path
import subprocess
import sys

TAGS = ["orchard", "harbor", "ridge", "meadow", "brook", "valley", "forest", "shore"]
WORKER = "--catalog"
SEEDS = (1, 2)
WAIT_SECONDS = 2
cache = {}


def catalog_key(tags):
    return ",".join(set(tags))


def main():
    if WORKER in sys.argv:
        print(catalog_key(TAGS))
        return
    keys = []
    for seed in SEEDS:
        environment = dict(os.environ, PYTHONHASHSEED=str(seed))
        result = subprocess.run([sys.executable, str(Path(__file__).resolve()), WORKER],
                                env=environment, capture_output=True, text=True,
                                check=True, timeout=WAIT_SECONDS)
        keys.append(result.stdout.strip())
    cache[keys[0]] = "catalog"
    print("cache hit", keys[1] in cache)


main()
