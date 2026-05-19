-- Enable required extensions on the bootstrap database.
-- This runs once when postgres initializes a fresh data directory.

CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS age;

-- AGE requires its schema on search_path for graph functions; load it now
-- so subsequent migrations and queries can reference age objects directly.
LOAD 'age';
SET search_path = ag_catalog, "$user", public;
