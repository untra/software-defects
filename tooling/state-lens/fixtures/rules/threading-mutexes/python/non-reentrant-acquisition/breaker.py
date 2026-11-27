import threading
m = threading.Lock()
def f():
    m.acquire()
    m.release()
    m.acquire()
    m.release()
