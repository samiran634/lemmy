-- Remove openrouter_api_key column from secret table
ALTER TABLE secret
DROP COLUMN openrouter_api_key;
