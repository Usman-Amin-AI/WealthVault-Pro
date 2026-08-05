INSERT INTO market_data_providers (id, name, description, url, priority, enabled)
VALUES 
  ('ALPACA', 'Alpaca', 'Live market data via Alpaca Markets', 'https://alpaca.markets/', 3, 0),
  ('POLYGON', 'Polygon.io', 'Live market data via Polygon.io', 'https://polygon.io/', 4, 0)
ON CONFLICT(id) DO NOTHING;
