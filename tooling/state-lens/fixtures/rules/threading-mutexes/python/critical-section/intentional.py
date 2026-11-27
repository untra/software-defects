import threading
m = threading.RLock()
def f():
    with m:
        with m:
            pass
