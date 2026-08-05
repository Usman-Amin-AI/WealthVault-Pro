CREATE TABLE live_prices_cache (
    symbol TEXT NOT NULL PRIMARY KEY,
    price TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    provider TEXT NOT NULL
);
