CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL,
    username_normalized TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT users_username_length CHECK (char_length(username) BETWEEN 3 AND 16)
);

CREATE TABLE sessions (
    token_hash BYTEA PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX sessions_user_id_idx ON sessions(user_id);
CREATE INDEX sessions_expiry_idx ON sessions(expires_at);

CREATE TABLE friend_requests (
    id UUID PRIMARY KEY,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT friend_requests_no_self CHECK (sender_id <> recipient_id)
);

-- Prevent duplicate requests in either direction for the same pair.
CREATE UNIQUE INDEX friend_requests_pair_unique
    ON friend_requests (LEAST(sender_id, recipient_id), GREATEST(sender_id, recipient_id));
CREATE INDEX friend_requests_recipient_idx ON friend_requests(recipient_id);

CREATE TABLE friendships (
    user_a UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_b UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT friendships_ordered_pair CHECK (user_a < user_b),
    PRIMARY KEY (user_a, user_b)
);

CREATE INDEX friendships_user_b_idx ON friendships(user_b);
