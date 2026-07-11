-- Default super-admin account.
-- IMPORTANT: password_hash below is a PLACEHOLDER. Generate a real bcrypt/argon2
-- hash for the password you intend to use and replace it before running in any
-- shared or production environment, e.g. (Node): bcrypt.hashSync('YourP@ssw0rd', 12)
-- NOTE: role assignment is no longer a column on this table -- see
-- 007_seed_user_roles.sql, which grants this user the 'admin' role via the
-- many-to-many user_roles table.
INSERT INTO users (name, username, email,  password_hash, is_active)
VALUES ('Super Admin', 'admin', 'admin@mail.local',
        '$2b$12$REPLACE.WITH.A.REAL.BCRYPT.HASH.................', true)
ON CONFLICT (email) WHERE deleted_at IS NULL DO NOTHING;
