from collections import deque

source = deque([1, 2], maxlen=3)
work = source.copy()
work.append(3)
assert list(work) == [1, 2, 3]
assert list(source) == [1, 2] and source.maxlen == 3, "source changed"
print("ok")
