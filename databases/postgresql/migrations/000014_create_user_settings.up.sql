-- Per-user key/value preferences (theme, locale, notification prefs, ...).
-- Unlike system_settings, always scoped to a single user_id and never
-- exposed cross-user -- see modules::user_setting.
CREATE TABLE user_settings (
    id          SERIAL PRIMARY KEY,
    user_id     INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key         VARCHAR(100) NOT NULL,
    value       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, key)
);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);

-- Reuses trigger_set_updated_at(), defined in migration 000003.
CREATE TRIGGER set_updated_at_user_settings
    BEFORE UPDATE ON user_settings
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();
