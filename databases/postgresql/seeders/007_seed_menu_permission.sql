-- Permission for managing the menu/navigation tree (added alongside the
-- `menu` module). Kept in its own file, rather than appended to
-- 001_seed_roles_and_permissions.sql, so it also lands correctly on
-- databases that were already seeded before this permission existed.
INSERT INTO permissions (name, description) VALUES
    ('menu.manage', 'Manage navigation menus and their permission mappings')
ON CONFLICT (name) DO NOTHING;

-- 001_seed_roles_and_permissions.sql grants admin every permission via a
-- CROSS JOIN, but that only ran once against the permissions that existed
-- at the time. Re-grant explicitly here so existing deployments pick up
-- 'menu.manage' too.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin' AND p.name = 'menu.manage'
ON CONFLICT DO NOTHING;
