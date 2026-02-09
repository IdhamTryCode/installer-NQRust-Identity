#!/bin/sh
# Idempotent: ensure analytics user and database exist.
# Run after northwind-db is up so that upgrades (existing volume) get the analytics role too.
# Uses POSTGRES_USER/POSTGRES_PASSWORD (demo/demo123) to connect as superuser.
# Uses ANALYTICS_DB_PASSWORD for the analytics user (must match password in PG_URL in .env).

set -e
# Use environment variables if set, otherwise psql defaults to local connection
# PGHOST, PGPORT, PGUSER, PGPASSWORD are typically provided by the environment
export PGUSER="${PGUSER:-${POSTGRES_USER:-demo}}"
export PGPASSWORD="${PGPASSWORD:-${POSTGRES_PASSWORD:-demo123}}"

# Password for user "analytics"; escape single quotes for SQL
ANALYTICS_PASS="${ANALYTICS_DB_PASSWORD:-analytics123}"
ANALYTICS_PASS_ESC=$(echo "$ANALYTICS_PASS" | sed "s/'/''/g")

# Use standard psql command; it will use PGHOST if set, otherwise local socket
PSQL="psql -v ON_ERROR_STOP=1"

echo "Setting up analytics user..."
$PSQL -d postgres <<EOSQL
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'analytics') THEN
    CREATE USER analytics WITH PASSWORD '${ANALYTICS_PASS_ESC}';
  END IF;
END \$\$;
ALTER USER analytics WITH PASSWORD '${ANALYTICS_PASS_ESC}';
ALTER USER analytics SET search_path TO public, analytics, "\$user";
EOSQL

# Create analytics database if not exists
echo "Ensuring analytics database exists..."
exists=$($PSQL -d postgres -t -A -c "SELECT 1 FROM pg_database WHERE datname = 'analytics'")
if [ -z "$exists" ]; then
  $PSQL -d postgres -c "CREATE DATABASE analytics OWNER analytics;"
fi

echo "Fixing permissions and ownership for Northwind database..."
$PSQL -d northwind <<EOSQL
  -- Transfer database ownership
  ALTER DATABASE northwind OWNER TO analytics;

  -- Fix for Postgres 15+: enable public schema usage
  ALTER SCHEMA public OWNER TO analytics;
  GRANT ALL ON SCHEMA public TO analytics;
  GRANT ALL ON SCHEMA public TO "${PGUSER}";
  GRANT ALL ON SCHEMA public TO PUBLIC;

  -- Grant CONNECT
  GRANT CONNECT ON DATABASE northwind TO analytics;
  GRANT CONNECT ON DATABASE northwind TO "${PGUSER}";

  -- Transfer ownership of all existing tables, sequences, and views to analytics
  DO \$\$ 
  DECLARE 
      r RECORD;
  BEGIN
      -- Tables
      FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public') LOOP
          EXECUTE 'ALTER TABLE public.' || quote_ident(r.tablename) || ' OWNER TO analytics';
      END LOOP;
      -- Sequences
      FOR r IN (SELECT sequence_name FROM information_schema.sequences WHERE sequence_schema = 'public') LOOP
          EXECUTE 'ALTER SEQUENCE public.' || quote_ident(r.sequence_name) || ' OWNER TO analytics';
      END LOOP;
      -- Views
      FOR r IN (SELECT viewname FROM pg_views WHERE schemaname = 'public') LOOP
          EXECUTE 'ALTER VIEW public.' || quote_ident(r.viewname) || ' OWNER TO analytics';
      END LOOP;
  END \$\$;

  -- Grant permissions (as a secondary measure)
  GRANT SELECT ON ALL TABLES IN SCHEMA public TO analytics;
  GRANT SELECT ON ALL TABLES IN SCHEMA public TO "${PGUSER}";
  GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO analytics;
  GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO "${PGUSER}";

  -- Future proofing
  ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO analytics;
  ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO "${PGUSER}";
EOSQL

echo "Fixing permissions for Analytics database..."
$PSQL -d analytics <<EOSQL
  GRANT ALL ON SCHEMA public TO analytics;
  GRANT ALL ON SCHEMA public TO "${PGUSER}";
  GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO analytics;
  GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO "${PGUSER}";
  ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO analytics;
  ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO "${PGUSER}";
EOSQL

echo "Database initialization and permission fix completed."
