-- Password history table: menyimpan hash password lama untuk mencegah reuse
CREATE TABLE IF NOT EXISTS user_password_histories (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_user_password_histories_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- Index untuk mempercepat query "ambil N riwayat password terakhir milik user"
CREATE INDEX IF NOT EXISTS idx_user_password_histories_user_id_created_at
    ON user_password_histories (user_id, created_at DESC);
