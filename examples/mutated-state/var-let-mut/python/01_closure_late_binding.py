def main():
    handlers = []
    for name in ("ada", "ben"):
        handlers.append(lambda: f"welcome email to {name}")
    for send in handlers:
        print(send())


main()
