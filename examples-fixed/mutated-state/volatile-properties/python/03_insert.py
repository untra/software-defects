source = [3, 1, 1, 2]
work = source.copy()
work.insert(1, 9)
assert work == [3, 9, 1, 1, 2]
assert source == [3, 1, 1, 2], "source changed"
print("ok")
