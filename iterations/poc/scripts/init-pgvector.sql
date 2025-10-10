-- Initialize pgvector extension for vector similarity search
CREATE EXTENSION IF NOT EXISTS "pgvector";

-- Initialize uuid-ossp extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Verify extensions are loaded
SELECT * FROM pg_extension WHERE extname IN ('pgvector', 'uuid-ossp');
