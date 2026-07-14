INSERT INTO permissions (name, description) VALUES
    ('file.upload', 'Upload new files'),
    ('file.read',   'List/view/download files'),
    ('file.delete', 'Delete (soft-delete) files')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND p.name IN ('file.upload', 'file.read', 'file.delete')
ON CONFLICT DO NOTHING;
