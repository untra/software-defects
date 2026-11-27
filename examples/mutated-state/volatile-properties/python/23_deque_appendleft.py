from collections import deque

source = deque([1, 2], maxlen=3)
work = source
work.appendleft(3)
assert list(work) == [3, 1, 2]
assert list(source) == [1, 2] and source.maxlen == 3, "source changed"
print("ok")
