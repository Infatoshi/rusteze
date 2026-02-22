-- Store image data as base64 directly in the database
-- For small images (<2MB), this avoids filesystem dependencies
ALTER TABLE attachments ADD COLUMN IF NOT EXISTS data TEXT;
