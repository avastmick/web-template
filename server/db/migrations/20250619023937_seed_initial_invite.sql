-- migrate:up
-- Seed initial invite for mick@kayshun.co
INSERT INTO user_invites (id, email, invited_by, invited_at)
VALUES (
    'invite_' || lower(hex(randomblob(16))),
    'mick@kayshun.co',
    'system',
    CURRENT_TIMESTAMP
);

-- migrate:down
-- Remove the initial invite
DELETE FROM user_invites WHERE email = 'mick@kayshun.co' AND invited_by = 'system';
