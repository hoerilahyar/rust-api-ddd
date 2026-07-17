-- New permission for the log_activity module. Reuses the same
-- "admin gets every permission" CROSS JOIN pattern from
-- 001_seed_roles_and_permissions.sql, so it's safe to re-run.
INSERT INTO permissions (name, description) VALUES
    ('log_activity.read', 'View general activity trail (log_activities)')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name = 'log_activity.read'
ON CONFLICT DO NOTHING;
