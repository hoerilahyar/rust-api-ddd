-- Extended per-user profile data, kept in its own table (not on `users`)
-- so the core identity/auth row stays lean and this can grow independently.
-- Strict 1:1 with `users` via the UNIQUE `user_id` FK; see
-- modules::user_profile.
CREATE TABLE user_profiles (
    id            SERIAL PRIMARY KEY,
    user_id       INT NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    phone         VARCHAR(30),
    address       TEXT,
    city          VARCHAR(100),
    country       VARCHAR(100),
    postal_code   VARCHAR(20),
    gender        VARCHAR(20),
    date_of_birth DATE,
    avatar_url    VARCHAR(255),
    website       VARCHAR(255),
    bio           TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_user_profiles_user_id ON user_profiles(user_id);

-- Reuses trigger_set_updated_at(), defined in migration 000003.
CREATE TRIGGER set_updated_at_user_profiles
    BEFORE UPDATE ON user_profiles
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();
