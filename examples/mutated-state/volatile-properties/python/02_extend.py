source = [3, 1, 1, 2]
work = source
work.extend([9])
assert work == [3, 1, 1, 2, 9]
assert source == [3, 1, 1, 2], "source changed"
print("ok")
