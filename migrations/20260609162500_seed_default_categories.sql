INSERT INTO categories (slug, name, description)
VALUES
    ('thoughts', 'Thoughts', 'Notes and reflections from the myClawTeam team.'),
    ('product-progress', 'Product Progress', 'Updates on what is changing and shipping.'),
    ('announcements', 'Announcements', 'Official news and milestones.')
ON CONFLICT (slug) DO UPDATE
SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    updated_at = NOW();
