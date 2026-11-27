cache = {}
def get(k):
    if k in cache:
        return cache[k]
    return None
