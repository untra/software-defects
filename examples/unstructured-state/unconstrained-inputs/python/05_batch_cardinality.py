MAX_BATCH_ITEMS = 3


def import_rows(rows):
    processed = 0
    for _ in rows:
        processed += 1
    return processed


try:
    print("processed", import_rows(["a", "b", "c", "d"]))
except ValueError as error:
    print(error)
