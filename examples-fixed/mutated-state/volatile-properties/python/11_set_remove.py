
source = {1, 2}
work = source.copy()
work.remove(1)
assert work == {2}
assert source == {1, 2}, "source changed"
print("ok")
