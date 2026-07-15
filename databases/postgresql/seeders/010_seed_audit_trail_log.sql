INSERT INTO permissions (name, description) VALUES
    ('audit_trail.read',   'List/view/download files'),
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name IN ('audit_trail.read')
ON CONFLICT DO NOTHING;
