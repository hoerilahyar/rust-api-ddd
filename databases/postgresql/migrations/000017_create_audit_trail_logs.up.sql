CREATE TABLE audit_trail_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         INT NULL,
    action          VARCHAR(50)  NOT NULL,
    entity_type     VARCHAR(100) NOT NULL,
    entity_id       UUID NULL,
    old_values      JSONB NULL,
    new_values      JSONB NULL,
    ip_address      VARCHAR(45) NULL,
    user_agent      TEXT NULL,
    description     TEXT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_audit_trail_logs_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);

CREATE INDEX idx_audit_trail_logs_user_id ON audit_trail_logs(user_id);
CREATE INDEX idx_audit_trail_logs_entity ON audit_trail_logs(entity_type, entity_id);
CREATE INDEX idx_audit_trail_logs_created_at ON audit_trail_logs(created_at);