-- Top-level menu items
INSERT INTO menus (parent_id, name, slug, path, icon, order_index) VALUES
    (NULL, 'Dashboard',       'dashboard',       '/dashboard',   'home',     1),
    (NULL, 'Administration',  'administration',  '/admin',       'settings', 2)
ON CONFLICT (slug) WHERE deleted_at IS NULL DO NOTHING;

-- Children of "Administration"
INSERT INTO menus (parent_id, name, slug, path, icon, order_index)
SELECT m.id, c.name, c.slug, c.path, c.icon, c.order_index
FROM menus m
JOIN (VALUES
    ('Users',            'admin.users',    '/admin/users',    'users',  1),
    ('Roles & Permissions', 'admin.roles', '/admin/roles',    'shield', 2),
    ('System Settings',  'admin.settings', '/admin/settings', 'cog',    3)
) AS c(name, slug, path, icon, order_index) ON true
WHERE m.slug = 'administration'
ON CONFLICT (slug) WHERE deleted_at IS NULL DO NOTHING;
