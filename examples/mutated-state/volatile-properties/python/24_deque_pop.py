from collections import deque

source = deque([1, 2], maxlen=3)
work = source
result = work.pop()
assert result == 2 and list(work) == [1]
assert list(source) == [1, 2] and source.maxlen == 3, "source changed"
print("ok")
