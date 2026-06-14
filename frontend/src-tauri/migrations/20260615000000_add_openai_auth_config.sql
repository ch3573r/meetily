-- Stores non-secret OpenAI auth-mode metadata.
-- API keys remain in openaiApiKey; OAuth client secrets and tokens are not stored here.
ALTER TABLE settings ADD COLUMN openAIAuthConfig TEXT;
