from io import StringIO

source = StringIO("abc")
work = source
result = work.seek(2)
assert result == 2 and work.getvalue() == "abc" and work.tell() == 2
assert source.getvalue() == "abc" and source.tell() == 0, "source changed"
print("ok")
