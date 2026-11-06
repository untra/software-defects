
source = {"a": 1, "b": 2}
work = source
work.update({"a": 9})
assert work == {"a": 9, "b": 2}
assert source == {"a": 1, "b": 2}, "source changed"
print("ok")
