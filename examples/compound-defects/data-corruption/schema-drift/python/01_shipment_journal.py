import json
from pathlib import Path
from tempfile import TemporaryDirectory

JOURNAL_VERSION = 2
WAREHOUSE_GRAMS = 500
KIOSK_KILOGRAMS = 2


def warehouse(path):
    with path.open("a") as handle:
        handle.write(json.dumps({"version": JOURNAL_VERSION, "weightGrams": WAREHOUSE_GRAMS}) + "\n")


def kiosk(path):
    with path.open("a") as handle:
        handle.write(json.dumps({"weight": KIOSK_KILOGRAMS}) + "\n")


def main():
    with TemporaryDirectory(prefix="shipments-") as directory:
        path = Path(directory) / "parcels.jsonl"
        warehouse(path)
        kiosk(path)
        for line in path.read_text().splitlines():
            parcel = json.loads(line)
            print("grams", parcel.get("weightGrams"))


main()
