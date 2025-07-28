-- Seed initial invite for mick@kayshun.co
INSERT OR IGNORE INTO user_invites(
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
