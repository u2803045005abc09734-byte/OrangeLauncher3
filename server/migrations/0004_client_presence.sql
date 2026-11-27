ALTER TABLE users
    ADD COLUMN client_seen_at TIMESTAMPTZ;

CREATE INDEX users_client_seen_at_idx
    ON users (client_seen_at)
    WHERE client_seen_at IS NOT NULL;
