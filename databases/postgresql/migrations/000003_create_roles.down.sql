DROP TRIGGER IF EXISTS set_updated_at_users ON users;
DROP TRIGGER IF EXISTS set_updated_at_roles ON roles;
DROP FUNCTION IF EXISTS trigger_set_updated_at();
DROP TABLE IF EXISTS roles;