-- Add openrouter_api_key column to secret table for secure storage
ALTER TABLE secret
ADD COLUMN openrouter_api_key VARCHAR;
