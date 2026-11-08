from collections import deque

source = deque([1, 2], maxlen=3)
work = source.copy()
result = work.popleft()
assert result == 1 and list(work) == [2]
assert list(source) == [1, 2] and source.maxlen == 3, "source changed"
print("ok")
