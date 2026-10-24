import ctypes
import mmap
import resource

FRAME_BYTES = mmap.PAGESIZE
SAMPLE = 23


def main():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    frame = mmap.mmap(-1, FRAME_BYTES)
    frame[0] = SAMPLE
    address = ctypes.addressof(ctypes.c_ubyte.from_buffer(frame))
    playback = ctypes.c_ubyte.from_address(address)
    print("playback requested", flush=True)
    frame.close()
    print("sample", playback.value)


main()
