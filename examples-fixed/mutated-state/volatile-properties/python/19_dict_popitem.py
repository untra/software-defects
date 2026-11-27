
source = {"a": 1, "b": 2}
work = source.copy()
result = work.popitem()
assert result == ("b", 2) and work == {"a": 1}
assert source == {"a": 1, "b": 2}, "source changed"
print("ok")
