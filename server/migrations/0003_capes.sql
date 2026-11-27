-- One published cape per verified Minecraft account. Updating a cape replaces the existing texture.
CREATE TABLE capes (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    texture BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT capes_texture_size_check
        CHECK (octet_length(texture) BETWEEN 1 AND 262144)
);

CREATE INDEX capes_updated_at_idx ON capes(updated_at);
