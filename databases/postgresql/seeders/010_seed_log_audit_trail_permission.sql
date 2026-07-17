INSERT INTO permissions (name, description) VALUES
    ('log_audit_trail.read',   'View ')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name IN ('log_audit_trail.read')
ON CONFLICT DO NOTHING;
