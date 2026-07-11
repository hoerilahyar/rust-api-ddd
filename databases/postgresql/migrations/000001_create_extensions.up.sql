-- Required extensions
-- pgcrypto: gen_random_uuid() for UUID primary keys
-- pg_trgm: trigram index support for fast ILIKE/name search
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
