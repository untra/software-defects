
source = {1, 2}
work = source.copy()
work.discard(1)
assert work == {2}
assert source == {1, 2}, "source changed"
print("ok")
