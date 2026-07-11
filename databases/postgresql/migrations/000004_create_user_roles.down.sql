ALTER TABLE users ADD COLUMN role_id INT;

-- Best-effort backfill: pick each user's earliest-assigned role
UPDATE users u
SET role_id = (
    SELECT ur.role_id FROM user_roles ur
    WHERE ur.user_id = u.id
    ORDER BY ur.assigned_at ASC
    LIMIT 1
);

ALTER TABLE users ADD CONSTRAINT users_role_id_fkey FOREIGN KEY (role_id) REFERENCES roles(id);
CREATE INDEX idx_users_role_id ON users(role_id);

DROP TABLE IF EXISTS user_roles;
