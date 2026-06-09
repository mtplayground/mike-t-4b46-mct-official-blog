CREATE TABLE media (
    id BIGSERIAL PRIMARY KEY,
    object_key TEXT NOT NULL UNIQUE,
    media_type TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT media_object_key_not_blank_check
        CHECK (char_length(btrim(object_key)) > 0),
    CONSTRAINT media_object_key_relative_check
        CHECK (object_key !~ '^/'),
    CONSTRAINT media_type_check
        CHECK (media_type IN ('image', 'video')),
    CONSTRAINT media_content_type_not_blank_check
        CHECK (char_length(btrim(content_type)) > 0),
    CONSTRAINT media_size_bytes_non_negative_check
        CHECK (size_bytes >= 0)
);

COMMENT ON COLUMN media.object_key IS
    'Relative Object Storage key under OBJECT_STORAGE_PREFIX. Signed read URLs are generated on demand and are not stored.';

CREATE INDEX media_type_created_at_idx
    ON media (media_type, created_at DESC, id DESC);

CREATE INDEX media_created_at_idx
    ON media (created_at DESC, id DESC);
