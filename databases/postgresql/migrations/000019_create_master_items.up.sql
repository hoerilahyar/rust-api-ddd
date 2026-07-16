-- Master table: Items
-- The actual reference values belonging to a group.
-- e.g. group 'GENDER' has items 'MALE', 'FEMALE'.
-- The `extra` JSONB column allows storing additional attributes per item
-- without altering the schema (e.g. color, icon, numeric weight, config flags).

CREATE TABLE master_items (
    id            BIGSERIAL PRIMARY KEY,
    group_id      BIGINT       NOT NULL REFERENCES master_groups (id) ON DELETE CASCADE,
    code          VARCHAR(50)  NOT NULL,          -- e.g. 'MALE', 'CREDIT_CARD'
    name          VARCHAR(150) NOT NULL,          -- e.g. 'Male', 'Credit Card'
    description   TEXT,
    extra         JSONB        NOT NULL DEFAULT '{}'::jsonb,
    sort_order    INT          NOT NULL DEFAULT 0,
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ,
    CONSTRAINT uq_master_items_group_code UNIQUE (group_id, code)
);

CREATE INDEX idx_master_items_group_id ON master_items (group_id);
CREATE INDEX idx_master_items_is_active ON master_items (is_active);
CREATE INDEX idx_master_items_deleted_at ON master_items (deleted_at);
CREATE INDEX idx_master_items_sort_order ON master_items (sort_order);
CREATE INDEX idx_master_items_extra ON master_items USING GIN (extra);

CREATE TRIGGER trg_master_items_updated_at
BEFORE UPDATE ON master_items
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
