INSERT INTO permissions (name, description) VALUES
    ('masters.manage',   'List/view/delete Master')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name IN ('masters.manage')
ON CONFLICT DO NOTHING;
