-- Metadata for uploaded/stored files. The actual bytes live on disk (or,
-- later, object storage) under `storage_path`; this table is the only
-- source of truth for what exists, who uploaded it, and whether it's been
-- (soft-)deleted -- mirrors the `deleted_at` convention used by `users`
-- and `menus` rather than the physical bytes being removed immediately.
CREATE TABLE files (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,

    -- Public identifier (never expose the internal `id` sequence in URLs).
    uuid UUID NOT NULL DEFAULT gen_random_uuid(),

    original_name  VARCHAR(255) NOT NULL,
    -- uuid-based name actually used on disk; never derived from user input,
    -- so path traversal / overwrite-another-file is not possible.
    stored_name     VARCHAR(255) NOT NULL,
    mime_type       VARCHAR(150) NOT NULL,
    size_bytes      BIGINT NOT NULL CHECK (size_bytes >= 0),
    storage_path    VARCHAR(500) NOT NULL,

    uploaded_by INT NULL,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ,

    CONSTRAINT fk_files_uploaded_by
        FOREIGN KEY (uploaded_by)
        REFERENCES users(id)
        ON DELETE SET NULL
);

CREATE UNIQUE INDEX idx_files_uuid_unique ON files(uuid);
CREATE UNIQUE INDEX idx_files_stored_name_unique ON files(stored_name) WHERE deleted_at IS NULL;
CREATE INDEX idx_files_uploaded_by ON files(uploaded_by);
CREATE INDEX idx_files_created_at ON files(created_at DESC);
-- Trigram index so `PaginationParams.search` can ILIKE against
-- original_name efficiently, same technique as `menus`/`users` searches.
CREATE INDEX idx_files_original_name_trgm ON files USING gin (original_name gin_trgm_ops);
