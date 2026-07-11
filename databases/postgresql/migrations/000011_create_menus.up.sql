-- Navigation/sidebar menu tree for the admin & staff apps. Self-referencing
-- parent_id supports nested menus (e.g. "Administration" -> "Users").
CREATE TABLE menus (
    id          SERIAL PRIMARY KEY,
    parent_id   INT REFERENCES menus(id) ON DELETE CASCADE,
    name        VARCHAR(150) NOT NULL,
    slug        VARCHAR(150) NOT NULL,
    path        VARCHAR(255),
    icon        VARCHAR(100),
    order_index INT NOT NULL DEFAULT 0,
    is_active   BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ
);

CREATE INDEX idx_menus_parent_id ON menus(parent_id);
CREATE UNIQUE INDEX idx_menus_slug_unique ON menus(slug) WHERE deleted_at IS NULL;
