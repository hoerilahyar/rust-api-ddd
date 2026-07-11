CREATE TABLE users (
    id            SERIAL PRIMARY KEY,
    name          VARCHAR(150) NOT NULL,
    username      VARCHAR(150) NOT NULL,
    email         VARCHAR(150) NOT NULL CHECK (length(username) >= 3),
    password_hash VARCHAR(255) NOT NULL,
    is_active     BOOLEAN NOT NULL DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at    TIMESTAMPTZ
);

-- Partial unique index: email & username is unique among non-deleted users only
CREATE UNIQUE INDEX idx_users_email_unique ON users(email) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_username_unique ON users(username) WHERE deleted_at IS NULL;
