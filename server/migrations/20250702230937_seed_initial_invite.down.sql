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
