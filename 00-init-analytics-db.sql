-- Create the analytics application database and user.
-- This runs before northwind.sql (alphabetical order in /docker-entrypoint-initdb.d/).
-- The northwind database is created automatically via POSTGRES_DB env var.

CREATE USER analytics WITH PASSWORD 'analytics123';
CREATE DATABASE analytics OWNER analytics;
GRANT ALL PRIVILEGES ON DATABASE analytics TO analytics;
