def stock_count(stock, item):
    count = stock.get(item, 0)
    return count

print("stock", stock_count({"pen": 4}, "pen"))
