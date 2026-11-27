from io import StringIO

source = StringIO("abc")
work = source
result = work.truncate(1)
assert result == 1 and work.getvalue() == "a" and work.tell() == 0
assert source.getvalue() == "abc" and source.tell() == 0, "source changed"
print("ok")
