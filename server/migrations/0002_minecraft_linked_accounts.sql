-- Link API identities to verified Minecraft profiles. Existing password-based
-- Orange accounts remain valid; new Minecraft users do not need an Orange password.
ALTER TABLE users
    ALTER COLUMN password_hash DROP NOT NULL;

ALTER TABLE users
    ADD COLUMN minecraft_uuid UUID;

CREATE UNIQUE INDEX users_minecraft_uuid_unique
    ON users (minecraft_uuid);

ALTER TABLE users
    ADD CONSTRAINT users_auth_method_required
    CHECK (password_hash IS NOT NULL OR minecraft_uuid IS NOT NULL);
