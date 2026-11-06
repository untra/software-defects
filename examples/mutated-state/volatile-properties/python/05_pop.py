source = [3, 1, 1, 2]
work = source
result = work.pop()
assert work == [3, 1, 1]
assert result == 2
assert source == [3, 1, 1, 2], "source changed"
print("ok")
