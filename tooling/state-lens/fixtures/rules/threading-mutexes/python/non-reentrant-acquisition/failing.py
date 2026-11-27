import threading
m = threading.Lock()
def f():
    m.acquire()
    m.acquire()
