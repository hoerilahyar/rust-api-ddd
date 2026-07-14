-- New permission for the activity_log module. Reuses the same
-- "admin gets every permission" CROSS JOIN pattern from
-- 001_seed_roles_and_permissions.sql, so it's safe to re-run.
INSERT INTO permissions (name, description) VALUES
    ('activity_log.read', 'View general activity trail (activity_logs)')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name = 'activity_log.read'
ON CONFLICT DO NOTHING;
