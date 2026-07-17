-- Permissions for the `role` and `permission` modules themselves.
--
-- Every other admin module got its own seeder when it was added
-- (007_seed_menu_permission.sql -> menu.manage, 008 -> activity_log.read,
-- 009 -> file.*, 010 -> audit_trail.read, 011 -> masters.manage, 012 ->
-- audit_auth.read), but `role.manage` and `permission.manage` were never
-- seeded at all even though:
--   - src/modules/role/presentation/handler.rs guards every route with
--     ensure_permission(&claims, "role.manage")
--   - src/modules/permission/presentation/handler.rs guards every route
--     with ensure_permission(&claims, "permission.manage")
-- With no row for either permission in the `permissions` table, nobody --
-- not even the seeded admin -- could ever be granted them, so the entire
-- /roles and /permissions APIs were unreachable (403) out of the box.
INSERT INTO permissions (name, description) VALUES
    ('role.manage',       'Manage roles and their permission assignments'),
    ('permission.manage', 'Manage the permission catalog')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name IN ('role.manage', 'permission.manage')
ON CONFLICT DO NOTHING;
