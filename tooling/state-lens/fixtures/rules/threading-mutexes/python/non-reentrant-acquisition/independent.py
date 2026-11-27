import threading
a = threading.Lock()
b = threading.Lock()
def f():
    with a:
        with b:
            pass
