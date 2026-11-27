
source = {1, 2}
work = source
result = work.pop()
assert result in {1, 2} and work == ({1, 2} - {result})
assert source == {1, 2}, "source changed"
print("ok")
