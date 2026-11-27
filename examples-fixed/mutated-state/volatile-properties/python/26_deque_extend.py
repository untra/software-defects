from collections import deque

source = deque([1, 2], maxlen=3)
work = source.copy()
work.extend([3, 4])
assert list(work) == [2, 3, 4]
assert list(source) == [1, 2] and source.maxlen == 3, "source changed"
print("ok")
