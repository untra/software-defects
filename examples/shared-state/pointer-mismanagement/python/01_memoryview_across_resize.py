def main():
    buf = bytearray(b"ada")
    view = memoryview(buf)
    buf.extend(b"ben")
    print(view[0])


main()
