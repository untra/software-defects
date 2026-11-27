source = [3, 1, 1, 2]
work = source.copy()
work.reverse()
assert work == [2, 1, 1, 3]
assert source == [3, 1, 1, 2], "source changed"
print("ok")
