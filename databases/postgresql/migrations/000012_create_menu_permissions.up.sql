-- Which permission unlocks visibility/access to a given menu item.
-- A menu with NO rows here is treated as visible to any authenticated user
-- (e.g. "Dashboard"). Access = user has >=1 role granting the mapped permission.
CREATE TABLE menu_permissions (
    menu_id       INT NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    permission_id INT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (menu_id, permission_id)
);

CREATE INDEX idx_menu_permissions_permission_id ON menu_permissions(permission_id);
