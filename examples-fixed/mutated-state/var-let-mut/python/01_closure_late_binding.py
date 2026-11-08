def main():
    handlers = []
    for name in ("ada", "ben"):
        handlers.append(lambda name=name: f"welcome email to {name}")
    for send in handlers:
        print(send())


main()
