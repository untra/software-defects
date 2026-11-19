def headers_for(token):
    return {"Authorization": token}


first = headers_for("a")
second = headers_for("b")
first["Authorization"] = ""
print(second)
