from pathlib import Path
from tempfile import TemporaryDirectory

CATALOG_NAME = "café"
WRITER_ENCODING = "latin1"
READER_ENCODING = "utf8"
catalog = {}


def main():
    with TemporaryDirectory(prefix="catalog-") as directory:
        path = Path(directory) / "name.txt"
        path.write_bytes(CATALOG_NAME.encode(WRITER_ENCODING))
        name = path.read_bytes().decode(READER_ENCODING, errors="replace")
        catalog["coffee"] = name
        print("name", catalog["coffee"])


main()
