def fib(n, memo={}):
    if n in memo:
        return memo[n]
    memo[n] = n if n < 2 else fib(n - 1) + fib(n - 2)
    return memo[n]


print(fib(10))
