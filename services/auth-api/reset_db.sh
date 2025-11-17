#!/bin/bash

# Drop and recreate database for exchange API
sudo -u postgres psql << EOF
DROP DATABASE IF EXISTS exchange_api;
CREATE DATABASE exchange_api;
GRANT ALL PRIVILEGES ON DATABASE exchange_api TO exchange_user;
\c exchange_api
GRANT ALL ON SCHEMA public TO exchange_user;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO exchange_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO exchange_user;
EOF

echo "Database reset complete!"
echo "Now run: cargo run"