INSERT INTO user_roles (user_id, role_id, assigned_at)
SELECT u.id, r.id, now()
FROM users u
JOIN roles r ON r.name = 'admin'
WHERE u.email = 'admin@mail.local'
ON CONFLICT DO NOTHING;
