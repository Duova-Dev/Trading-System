import requests

base_endpoint = "https://api.binance.us"

other_endpoints = ["https://api1.binance.us",
                   "https://api2.binance.us", "https://api3.binance.us"]

api_key = "xzRECFLAt57N81wBIJKpu5QUrqeTtZyGShABhcQ2yESasrBndEpk8aZ8xLsEgDXU"

session = requests.session()
session.headers.update({'X-MBX-APIKEY': api_key})


def test_connectivity():
    return session.get(base_endpoint + "/api/v3/ping")


def get_server_time():
    return session.get(base_endpoint + "/api/v3/time")


def get_exchange_info():
    return session.get(base_endpoint + "/api/v3/exchangeInfo")


def get_orderbook(symbol, limit=100):
    params = {'symbol': symbol, 'limit': limit}
    return session.get(base_endpoint + "/api/v3/depth", params=params)


def get_recent_trades(symbol, limit=500):
    params = params = {'symbol': symbol, 'limit': limit}
    return session.get(base_endpoint + "/api/v3/trades", params=params)


def get_historical_trades(symbol, limit=500, fromId=-1):
    params = {'symbol': symbol, 'limit': limit}
    if fromId != -1:
        params['fromId'] = fromId
    return session.get(base_endpoint + "/api/v3/historicalTrades", params=params)


def get_aggregate_trades(symbol, startTime=-1, endTime=-1, limit=500, fromId=-1):
    params = {'symbol': symbol, 'limit': limit}
    if startTime != -1:
        params['startTime'] = startTime

    if endTime != -1:
        params['endTime'] = endTime

    if fromId != -1:
        params['fromId'] = fromId

    return session.get(base_endpoint + "/api/v3/aggTrades", params=params)


def get_klines(symbol, interval, startTime=-1, endTime=-1, limit=500):
    params = {'symbol': symbol, 'interval': interval, 'limit': limit}
    if startTime != -1:
        params['startTime'] = startTime

    if endTime != -1:
        params['endTime'] = endTime

    return session.get(base_endpoint + "/api/v3/klines", params=params)


def get_avg_price(symbol):
    params = {'symbol': symbol}
    return session.get(base_endpoint + "/api/v3/avgPrice", params=params)


def get_24hr_change(symbol):
    params = {'symbol': symbol}
    return session.get(base_endpoint + "/api/v3/ticker/24hr", params=params)


def get_latest_price(symbol):
    params = {'symbol': symbol}
    return session.get(base_endpoint + "/api/v3/ticker/price", params=params)


def get_best_from_orderbook(symbol=""):
    params = {}
    if symbol:
        params['symbol'] = symbol
    return session.get(base_endpoint + "/api/v3/ticker/bookTicker", params=params)


# print(get_klines(
#     "ETHUSDT", interval = "1m", startTime = 1613368440333, endTime = 1613368564524).json())
# print(get_historical_trades("ETHUSDT").json())

print(get_best_from_orderbook().json())
