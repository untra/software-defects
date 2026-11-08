from io import StringIO

source = StringIO("abc")
work = StringIO(source.getvalue())
work.seek(source.tell())
result = work.write("Z")
assert result == 1 and work.getvalue() == "Zbc" and work.tell() == 1
assert source.getvalue() == "abc" and source.tell() == 0, "source changed"
print("ok")
