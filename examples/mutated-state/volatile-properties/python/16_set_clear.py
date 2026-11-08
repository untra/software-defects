
source = {1, 2}
work = source
work.clear()
assert work == set()
assert source == {1, 2}, "source changed"
print("ok")
