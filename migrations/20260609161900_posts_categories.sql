CREATE TABLE categories (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT categories_slug_format_check
        CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    CONSTRAINT categories_name_not_blank_check
        CHECK (char_length(btrim(name)) > 0)
);

CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    category_id BIGINT NOT NULL
        REFERENCES categories(id)
        ON UPDATE CASCADE
        ON DELETE RESTRICT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT posts_title_not_blank_check
        CHECK (char_length(btrim(title)) > 0),
    CONSTRAINT posts_slug_format_check
        CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    CONSTRAINT posts_status_check
        CHECK (status IN ('draft', 'published')),
    CONSTRAINT posts_published_at_status_check
        CHECK (
            (status = 'draft' AND published_at IS NULL)
            OR (status = 'published' AND published_at IS NOT NULL)
        )
);

CREATE INDEX posts_category_id_idx
    ON posts (category_id);

CREATE INDEX posts_status_published_at_idx
    ON posts (status, published_at DESC, id DESC);

CREATE INDEX posts_category_status_published_at_idx
    ON posts (category_id, status, published_at DESC, id DESC);

CREATE INDEX posts_created_at_idx
    ON posts (created_at DESC, id DESC);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER categories_set_updated_at
BEFORE UPDATE ON categories
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER posts_set_updated_at
BEFORE UPDATE ON posts
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
