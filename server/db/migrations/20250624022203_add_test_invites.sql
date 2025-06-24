-- migrate:up
-- Add test invites for test@example.com and avastmick@outlook.com
INSERT INTO user_invites (id, email, invited_by, invited_at)
VALUES
    (
        'invite_' || lower(hex(randomblob(16))),
        'test@example.com',
        'system',
        CURRENT_TIMESTAMP
    ),
    (
        'invite_' || lower(hex(randomblob(16))),
        'avastmick@outlook.com',
        'system',
        CURRENT_TIMESTAMP
    );

-- migrate:down
-- Remove the test invites
DELETE FROM user_invites WHERE email IN ('test@example.com', 'avastmick@outlook.com') AND invited_by = 'system';
