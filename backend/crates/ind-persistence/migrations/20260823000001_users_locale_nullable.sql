ALTER TABLE users ALTER COLUMN locale DROP NOT NULL;
ALTER TABLE users ALTER COLUMN locale DROP DEFAULT;

-- Bare English was the registration default; explicit UI choices used regional tags.
UPDATE users SET locale = NULL WHERE locale = 'en';
