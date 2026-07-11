-- Generic admin-configurable key/value store (upload limits, default storage provider, pagination defaults, etc.)
CREATE TABLE system_settings (
    id          SERIAL PRIMARY KEY,
    key         VARCHAR(100) NOT NULL UNIQUE,
    value       TEXT,
    description VARCHAR(255),
    updated_by  INT REFERENCES users(id),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
