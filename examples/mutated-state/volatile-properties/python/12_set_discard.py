
source = {1, 2}
work = source
work.discard(1)
assert work == {2}
assert source == {1, 2}, "source changed"
print("ok")
