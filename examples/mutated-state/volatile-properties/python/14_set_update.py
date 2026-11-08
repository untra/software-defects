
source = {1, 2}
work = source
work.update({3})
assert work == {1, 2, 3}
assert source == {1, 2}, "source changed"
print("ok")
