-- Master table: Groups
-- Defines a "category of reference data" — e.g. GENDER, PAYMENT_METHOD,
-- DOCUMENT_TYPE, ORDER_STATUS. This is the top level of the generic
-- lookup/reference pattern: one Group has many Items (master_items).
-- This pattern lets new reference data types be added by inserting rows,
-- without creating a new table for every "type" or "status" concept.

CREATE TABLE master_groups (
    id            BIGSERIAL PRIMARY KEY,
    code          VARCHAR(50)  NOT NULL UNIQUE,   -- e.g. 'GENDER', 'PAYMENT_METHOD'
    name          VARCHAR(150) NOT NULL,          -- e.g. 'Gender', 'Payment Method'
    description   TEXT,
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at    TIMESTAMPTZ
);

CREATE INDEX idx_master_groups_is_active ON master_groups (is_active);
CREATE INDEX idx_master_groups_deleted_at ON master_groups (deleted_at);

CREATE TRIGGER trg_master_groups_updated_at
BEFORE UPDATE ON master_groups
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
