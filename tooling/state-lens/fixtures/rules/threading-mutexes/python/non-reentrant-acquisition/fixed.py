import threading
m = threading.Lock()
def f():
    with m:
        pass
