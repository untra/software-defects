source = [3, 1, 1, 2]
work = source
work.remove(1)
assert work == [3, 1, 2]
assert source == [3, 1, 1, 2], "source changed"
print("ok")
