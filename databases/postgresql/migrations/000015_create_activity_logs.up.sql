-- General-purpose, append-only activity trail across every module (not just
-- login attempts -- see user_login_logs / AuditRecorder for that narrower case).
-- ip_address is VARCHAR, matching user_login_logs, so it binds as plain TEXT
-- with sqlx (the "postgres" feature does not include the "ipnetwork" crate
-- needed to bind Rust values against a native INET column).
CREATE TABLE activity_logs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    user_id INT NULL,

    activity VARCHAR(100) NOT NULL,
    module VARCHAR(100) NOT NULL,

    resource_type VARCHAR(100),
    resource_id VARCHAR(100),

    method VARCHAR(10) NOT NULL,
    path VARCHAR(255) NOT NULL,

    description TEXT,

    ip_address VARCHAR(45),
    user_agent TEXT,

    status_code SMALLINT,

    trace_id UUID,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_activity_logs_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);

CREATE INDEX idx_activity_logs_user_id
    ON activity_logs(user_id);

CREATE INDEX idx_activity_logs_activity
    ON activity_logs(activity);

CREATE INDEX idx_activity_logs_module
    ON activity_logs(module);

CREATE INDEX idx_activity_logs_resource
    ON activity_logs(resource_type, resource_id);

CREATE INDEX idx_activity_logs_created_at
    ON activity_logs(created_at DESC);

CREATE INDEX idx_activity_logs_trace_id
    ON activity_logs(trace_id);
