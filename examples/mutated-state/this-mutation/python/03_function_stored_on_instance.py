class Report:
    def __init__(self):
        self.title = banner

    def render(self):
        return self.title()


def banner(self):
    return f"report for {self.owner}"


def main():
    report = Report()
    report.owner = "ada"
    print(report.render())


main()
