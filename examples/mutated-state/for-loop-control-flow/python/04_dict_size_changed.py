def main():
    ages = {"ada": 30, "ben": 31}
    for name in ages:
        if ages[name] > 30:
            del ages[name]
    print(ages)


main()
