-- migrate:up
-- Seed initial invite for mick@kayshun.co
INSERT INTO user_invites(
    id,
    email,
    invited_by,
    invited_at,
    expires_at
)
VALUES(
    'invite_'||LOWER(hex(randomblob(16))),
    'mick@kayshun.co',
    'system',
    CURRENT_TIMESTAMP,
    datetime('now', '+12 months')
),
(
    'invite_'||LOWER(hex(randomblob(16))),
    'test@example.com',
    'system',
    CURRENT_TIMESTAMP,
    datetime('now', '+12 months')
),
(
    'invite_'||LOWER(hex(randomblob(16))),
    'avastmick@outlook.com',
    'system',
    CURRENT_TIMESTAMP,
    datetime('now', '+12 months')
);
-- migrate:down
-- Remove the initial invite
DELETE FROM user_invites
WHERE
email IN
(
    'mick@kayshun.co',
    'test@example.com',
    'avastmick@outlook.com'
)
AND invited_by = 'system';
