#!/bin/sh
# Idempotent: ensure analytics user and database exist.
# Run after northwind-db is up so that upgrades (existing volume) get the analytics role too.
# Uses POSTGRES_USER/POSTGRES_PASSWORD (demo/demo123) to connect as superuser.
# Uses ANALYTICS_DB_PASSWORD for the analytics user (must match password in PG_URL in .env).

set -e
export PGHOST="${PGHOST:-northwind-db}"
export PGPORT="${PGPORT:-5432}"
export PGUSER="${PGUSER:-${POSTGRES_USER:-demo}}"
export PGPASSWORD="${PGPASSWORD:-${POSTGRES_PASSWORD:-demo123}}"

# Password for user "analytics"; escape single quotes for SQL
ANALYTICS_PASS="${ANALYTICS_DB_PASSWORD:-analytics123}"
ANALYTICS_PASS_ESC=$(echo "$ANALYTICS_PASS" | sed "s/'/''/g")

# Create analytics user if not exists
psql -d postgres -v ON_ERROR_STOP=1 <<EOSQL
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'analytics') THEN
    CREATE USER analytics WITH PASSWORD '${ANALYTICS_PASS_ESC}';
  END IF;
END \$\$;
EOSQL

# Always set password so it matches .env (fixes "password authentication failed" if it was wrong)
psql -d postgres -v ON_ERROR_STOP=1 -c "ALTER USER analytics WITH PASSWORD '${ANALYTICS_PASS_ESC}';"

# Create analytics database if not exists (CREATE DATABASE cannot run inside a transaction block)
exists=$(psql -d postgres -t -A -c "SELECT 1 FROM pg_database WHERE datname = 'analytics'")
if [ -z "$exists" ]; then
  psql -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE analytics OWNER analytics;"
  psql -d postgres -v ON_ERROR_STOP=1 -c "GRANT ALL PRIVILEGES ON DATABASE analytics TO analytics;"
fi

# Let analytics user read Northwind demo data (so UI/engine can query products, orders, etc.)
psql -d postgres -v ON_ERROR_STOP=1 -c "GRANT CONNECT ON DATABASE northwind TO analytics;"
psql -d northwind -v ON_ERROR_STOP=1 -c "GRANT USAGE ON SCHEMA public TO analytics;"
psql -d northwind -v ON_ERROR_STOP=1 -c "GRANT SELECT ON ALL TABLES IN SCHEMA public TO analytics;"
