import os
import tempfile


def main():
    descriptor, path = tempfile.mkstemp()
    os.close(descriptor)
    try:
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("ana,ben")
        mode = "w"
        try:
            with open(path, mode, encoding="utf-8") as handle:
                raise RuntimeError("disk full")
                handle.write("ada")
        except RuntimeError:
            pass
        with open(path, encoding="utf-8") as handle:
            print(repr(handle.read()))
    finally:
        os.remove(path)


main()
