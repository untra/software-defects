FIRST_TENANT = "orchard"
pool = [{"tenant": ""}]


def select_tenant(connection, tenant):
    connection["tenant"] = tenant


def release(connection):
    pool.append(connection)


def main():
    first = pool.pop()
    select_tenant(first, FIRST_TENANT)
    release(first)
    second = pool.pop()
    print("tenant", second["tenant"])
    release(second)


main()
