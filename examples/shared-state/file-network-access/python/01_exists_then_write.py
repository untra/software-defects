import os
import tempfile
import threading


def main():
    descriptor, path = tempfile.mkstemp()
    os.close(descriptor)
    try:
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("old")
        first_checked = threading.Event()
        second_done = threading.Event()

        def replace():
            first_checked.wait()
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("ben")
            second_done.set()

        def writer():
            if os.path.exists(path):
                first_checked.set()
                second_done.wait()
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write("ada")

        other = threading.Thread(target=replace)
        other.start()
        writer()
        other.join()
        with open(path, encoding="utf-8") as handle:
            print(handle.read())
    finally:
        os.remove(path)


main()
