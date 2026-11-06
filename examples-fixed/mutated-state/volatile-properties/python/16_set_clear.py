
source = {1, 2}
work = source.copy()
work.clear()
assert work == set()
assert source == {1, 2}, "source changed"
print("ok")
