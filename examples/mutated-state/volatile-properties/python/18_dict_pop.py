
source = {"a": 1, "b": 2}
work = source
result = work.pop("a")
assert result == 1 and work == {"b": 2}
assert source == {"a": 1, "b": 2}, "source changed"
print("ok")
