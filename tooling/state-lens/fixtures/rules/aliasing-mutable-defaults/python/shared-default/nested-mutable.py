def record(event, log={"events": []}):
    log["events"].append(event)
    return log


record("a")
print(record("b"))
