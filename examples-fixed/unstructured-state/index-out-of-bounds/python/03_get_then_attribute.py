def main():
    ages = {"ada": 30}
    found = ages.get("michael")
    print(found + 1 if found is not None else "age unavailable")


main()
