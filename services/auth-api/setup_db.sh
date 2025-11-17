#!/bin/bash

# Create database and user for exchange API
sudo -u postgres psql << EOF
CREATE DATABASE exchange_api;
CREATE USER exchange_user WITH PASSWORD 'exchange_password';
GRANT ALL PRIVILEGES ON DATABASE exchange_api TO exchange_user;
\c exchange_api
GRANT ALL ON SCHEMA public TO exchange_user;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO exchange_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO exchange_user;
EOF

echo "Database setup complete!"
echo "Database URL: postgres://exchange_user:exchange_password@localhost/exchange_api"