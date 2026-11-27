source = [3, 1, 1, 2]
work = source.copy()
work.sort()
assert work == [1, 1, 2, 3]
assert source == [3, 1, 1, 2], "source changed"
print("ok")
