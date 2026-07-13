-- Default super-admin account.
-- IMPORTANT: password_hash below is a PLACEHOLDER and is NOT a valid hash for
-- this app -- it's bcrypt-shaped text, but the app verifies passwords with
-- Argon2 (see AuthServiceImpl::verify_password), which will fail to even
-- parse it. Logging in as this seeded user WILL fail with a 500 until you
-- replace it. Generate a real hash with the app's own Argon2 config:
--   cargo run --example hash_password -- "YourP@ssw0rd"
-- then either edit the hash below before first running the seeders, or if
-- this has already been seeded, update the existing row directly:
--   UPDATE users SET password_hash = '<hash from the command above>'
--   WHERE email = 'admin@mail.local';
-- NOTE: role assignment is no longer a column on this table -- see
-- 004_seed_user_roles.sql, which grants this user the 'admin' role via the
-- many-to-many user_roles table.
INSERT INTO users (name, username, email,  password_hash, is_active)
VALUES ('Super Admin', 'admin', 'admin@mail.local',
        '$2b$12$REPLACE.WITH.A.REAL.BCRYPT.HASH.................', true)
ON CONFLICT (email) WHERE deleted_at IS NULL DO NOTHING;
