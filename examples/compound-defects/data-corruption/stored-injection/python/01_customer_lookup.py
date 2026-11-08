from pathlib import Path
from tempfile import TemporaryDirectory

CUSTOMER_NAME = "O'Neil"


def main():
    with TemporaryDirectory(prefix="customers-") as directory:
        path = Path(directory) / "name.txt"
        path.write_text(CUSTOMER_NAME)
        name = path.read_text()
        query = "SELECT id FROM customers WHERE name = '" + name + "'"
        print(query)


main()
