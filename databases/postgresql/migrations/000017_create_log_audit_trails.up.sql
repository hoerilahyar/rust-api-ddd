CREATE TABLE log_audit_trails (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    uuid UUID       NOT NULL DEFAULT gen_random_uuid(),
    user_id         INT NULL,
    action          VARCHAR(50)  NOT NULL,
    entity_type     VARCHAR(100) NOT NULL,
    entity_id       TEXT NULL,
    old_values      JSONB NULL,
    new_values      JSONB NULL,
    ip_address      VARCHAR(45) NULL,
    user_agent      TEXT NULL,
    description     TEXT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_log_audit_trails_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);

CREATE INDEX idx_log_audit_trails_user_id ON log_audit_trails(user_id);
CREATE INDEX idx_log_audit_trails_entity ON log_audit_trails(entity_type, entity_id);
CREATE INDEX idx_log_audit_trails_created_at ON log_audit_trails(created_at);