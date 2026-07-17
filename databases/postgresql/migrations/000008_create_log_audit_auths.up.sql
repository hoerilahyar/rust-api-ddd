CREATE TABLE log_audit_auths (
    id              BIGSERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id) ON DELETE SET NULL,
    email_attempted VARCHAR(150),
    ip_address      VARCHAR(45),
    user_agent      VARCHAR(255),
    status          VARCHAR(20) NOT NULL, -- 'success' | 'failed'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_log_audit_auths_user_id ON log_audit_auths(user_id);
CREATE INDEX idx_log_audit_auths_created_at ON log_audit_auths(created_at);
