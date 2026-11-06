
source = {"a": 1, "b": 2}
work = source.copy()
result = work.setdefault("c", 3)
assert result == 3 and work == {"a": 1, "b": 2, "c": 3}
assert source == {"a": 1, "b": 2}, "source changed"
print("ok")
