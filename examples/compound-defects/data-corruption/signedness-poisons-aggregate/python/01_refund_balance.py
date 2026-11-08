import asyncio
import ctypes

OPENING_BALANCE = 100
POSTING_CAPACITY = 1
REFUND = -1
balance = OPENING_BALANCE
in_flight = 0

async def post(refund, ready):
    global balance, in_flight
    if in_flight == POSTING_CAPACITY:
        print("ledger busy")
        return
    in_flight += 1
    try:
        await ready.wait()
        credit = ctypes.c_uint32(refund).value
        balance += credit
    finally:
        in_flight -= 1

async def main():
    ready = asyncio.Event()
    first = asyncio.create_task(post(REFUND, ready))
    await asyncio.sleep(0)
    await post(REFUND, ready)
    ready.set()
    await first
    print("balance", balance)

asyncio.run(main())
