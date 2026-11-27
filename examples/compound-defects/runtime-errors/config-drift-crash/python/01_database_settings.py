import os

DATABASE_ADDRESS = "postgres://warehouse"
PREFIX_LENGTH = 8
KEYS = ("DB_URL", "DATABASE_URL")


def configure():
    os.environ["DB_URL"] = DATABASE_ADDRESS


def address_prefix():
    return os.environ.get("DATABASE_URL")[:PREFIX_LENGTH]


def main():
    previous = {key: os.environ.get(key) for key in KEYS}
    try:
        os.environ.pop("DATABASE_URL", None)
        configure()
        print("database", address_prefix())
    finally:
        for key, value in previous.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


main()
