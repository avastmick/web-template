CREATE TABLE issues (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    type TEXT NOT NULL CHECK (type IN ('feature', 'enhancement', 'defect', 'task')),
    status TEXT NOT NULL CHECK (status IN ('backlog', 'current_release', 'doing', 'review', 'done')),
    priority TEXT NOT NULL CHECK (priority IN ('critical', 'high', 'medium', 'low')),
    size_estimate INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_issues_board_id ON issues(board_id);
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_priority ON issues(priority);
CREATE INDEX idx_issues_created_by ON issues(created_by);
