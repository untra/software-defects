cache = {}
def get(k):
    if k in cache:
        return cache[k]
    cache[k] = None
    return None
