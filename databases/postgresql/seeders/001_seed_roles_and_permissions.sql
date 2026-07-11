-- Roles
INSERT INTO roles (name, description) VALUES
    ('admin', 'Full access: manage users, products, categories, inventory, and settings')
ON CONFLICT (name) DO NOTHING;

-- Permissions
INSERT INTO permissions (name, description) VALUES
    ('user.manage',          'Manage users and role assignments'),
    ('audit.read',           'View audit history'),
    ('settings.manage',      'Manage system settings')
ON CONFLICT (name) DO NOTHING;

-- admin gets every permission
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
ON CONFLICT DO NOTHING;
