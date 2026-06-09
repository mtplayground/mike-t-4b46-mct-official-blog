CREATE TABLE newsletter_subscribers (
    id BIGSERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT newsletter_subscribers_email_not_blank_check
        CHECK (char_length(btrim(email)) > 0),
    CONSTRAINT newsletter_subscribers_email_format_check
        CHECK (email ~* '^[^@[:space:]]+@[^@[:space:]]+\.[^@[:space:]]+$')
);

CREATE UNIQUE INDEX newsletter_subscribers_email_unique_idx
    ON newsletter_subscribers (lower(email));

CREATE INDEX newsletter_subscribers_created_at_idx
    ON newsletter_subscribers (created_at DESC, id DESC);
