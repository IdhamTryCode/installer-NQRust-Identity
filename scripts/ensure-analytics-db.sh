#!/bin/sh
# Idempotent: ensure analytics user and database exist.
# Run after northwind-db is up so that upgrades (existing volume) get the analytics role too.
# Uses POSTGRES_USER/POSTGRES_PASSWORD (demo/demo123) to connect as superuser.

set -e
export PGHOST="${PGHOST:-northwind-db}"
export PGPORT="${PGPORT:-5432}"
export PGUSER="${PGUSER:-${POSTGRES_USER:-demo}}"
export PGPASSWORD="${PGPASSWORD:-${POSTGRES_PASSWORD:-demo123}}"

# Create analytics user if not exists
psql -d postgres -v ON_ERROR_STOP=1 <<'EOSQL'
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'analytics') THEN
    CREATE USER analytics WITH PASSWORD 'analytics123';
  END IF;
END $$;
EOSQL

# Create analytics database if not exists (must run outside transaction)
exists=$(psql -d postgres -t -A -c "SELECT 1 FROM pg_database WHERE datname = 'analytics'")
if [ -z "$exists" ]; then
  psql -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE analytics OWNER analytics; GRANT ALL PRIVILEGES ON DATABASE analytics TO analytics;"
fi
