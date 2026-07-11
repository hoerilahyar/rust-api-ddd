CREATE TABLE user_login_logs (
    id              BIGSERIAL PRIMARY KEY,
    user_id         INT REFERENCES users(id) ON DELETE SET NULL,
    email_attempted VARCHAR(150),
    ip_address      VARCHAR(45),
    user_agent      VARCHAR(255),
    status          VARCHAR(20) NOT NULL, -- 'success' | 'failed'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_user_login_logs_user_id ON user_login_logs(user_id);
CREATE INDEX idx_user_login_logs_created_at ON user_login_logs(created_at);
