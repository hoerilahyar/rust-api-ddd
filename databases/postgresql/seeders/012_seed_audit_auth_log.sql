-- Permission for the audit_auth_log module (login-attempt / authentication
-- audit trail), analogous to 010_seed_audit_trail_log.sql for
-- audit_trail_log.
--
-- Without this seeder, `audit_auth_log::presentation::handler` guards both
-- of its routes with `ensure_permission(&claims, "audit_auth.read")`, but
-- no seeder ever inserted that permission or granted it to `admin` -- so
-- even the seeded admin user gets 403 Forbidden on GET /audit-auth/logs
-- and GET /audit-auth/logs/:id, with no way to grant a permission that
-- doesn't exist in the `permissions` table.
--
-- The 'audit.read' row from 001_seed_roles_and_permissions.sql predates
-- the audit module being split into audit_auth_log/audit_trail_log and is
-- never checked anywhere in code; it's left in place (not deleted) since
-- this is a seed script that may already have run against a live
-- database, but it should be treated as dead/unused going forward.
INSERT INTO permissions (name, description) VALUES
    ('audit_auth.read', 'View authentication audit history (login attempts)')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name = 'audit_auth.read'
ON CONFLICT DO NOTHING;
