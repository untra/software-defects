from collections import deque

source = deque([1, 2], maxlen=3)
work = source
work.rotate(1)
assert list(work) == [2, 1]
assert list(source) == [1, 2] and source.maxlen == 3, "source changed"
print("ok")
