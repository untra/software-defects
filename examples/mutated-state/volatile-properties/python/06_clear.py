source = [3, 1, 1, 2]
work = source
work.clear()
assert work == []
assert source == [3, 1, 1, 2], "source changed"
print("ok")
