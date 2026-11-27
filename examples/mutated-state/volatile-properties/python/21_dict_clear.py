
source = {"a": 1, "b": 2}
work = source
work.clear()
assert work == {}
assert source == {"a": 1, "b": 2}, "source changed"
print("ok")
