import pickle
def load(raw, other):
    if isinstance(other, str):
        return pickle.loads(raw)
