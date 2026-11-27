cache = {}
other = {}
def get(k):
    other.pop(k, None)
    if k in cache:
        return cache[k]
    cache[k] = None
    return None
