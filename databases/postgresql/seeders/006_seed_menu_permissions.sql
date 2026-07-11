-- Maps each menu (by slug) to the permission required to see/access it.
-- "dashboard" is intentionally left unmapped -> visible to any authenticated user.
INSERT INTO menu_permissions (menu_id, permission_id)
SELECT m.id, p.id
FROM (VALUES
    ('administration',           'user.manage'),
    ('admin.users',              'user.manage'),
    ('admin.roles',              'user.manage'),
    ('admin.settings',           'settings.manage')
) AS mp(menu_slug, perm_name)
JOIN menus m ON m.slug = mp.menu_slug
JOIN permissions p ON p.name = mp.perm_name
ON CONFLICT DO NOTHING;
