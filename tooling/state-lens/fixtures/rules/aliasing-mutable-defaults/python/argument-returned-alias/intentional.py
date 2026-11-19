settings = {"debug": False}


def configure(options):
    return options


configure(settings)["debug"] = True
print(settings)
