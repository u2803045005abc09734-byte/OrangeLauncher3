# Orange Server

The first Orange-owned API backend for OrangeLauncher3. It is intentionally a separate Rust workspace under `server/`, so it does not become a member of the launcher's Cargo workspace.

## Included in this first backend milestone

- Username/password registration and login.
- Argon2id password hashing; plaintext passwords are never stored.
- Opaque bearer sessions whose hashes are stored in PostgreSQL.
- Session expiration and logout/revocation.
- Username prefix search.
- Friend requests, incoming request list, accept/reject, friends list and remove.
- Microsoft Minecraft profile verification; the Minecraft access token is exchanged once and never stored.
- Free cape publishing for verified Minecraft accounts: one cape per player, with uploads replacing the existing cape.
- Cape uploads are validated 64x32 PNG files, limited to 256 KiB, and stored in PostgreSQL.
- Public cape texture delivery by Minecraft UUID.
- Database migrations run automatically when the API starts.
- Docker Compose for a local PostgreSQL database and API.

Chat messages, presence and parties are not implemented in this backend yet. The cape API is ready for an Orange Client integration, but the in-game mod still needs to call these endpoints to provide the cape editor, equip display and tab-list badge.

## Run locally with Docker

From the repository root:

```sh
cd server
docker compose up --build
```

The API listens at `http://localhost:8080`. Check it with:

```sh
curl http://localhost:8080/health
```

For running without Docker, create a PostgreSQL database, copy `.env.example` to `.env`, set `DATABASE_URL`, then run:

```sh
cargo run
```

## API

All endpoints use JSON except where noted.

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/health` | Database readiness |
| POST | `/v1/auth/register` | Create an Orange account |
| POST | `/v1/auth/login` | Log in and create a session |
| GET | `/v1/me` | Current account |
| POST | `/v1/auth/logout` | Revoke current session |
| GET | `/v1/users/search?q=abc` | Search Orange usernames |
| GET | `/v1/friends` | List friends |
| GET | `/v1/friends/requests` | List incoming requests |
| POST | `/v1/friends/requests` | Send a request using `{"username":"Example"}` |
| POST | `/v1/friends/requests/{id}/accept` | Accept a request |
| POST | `/v1/friends/requests/{id}/reject` | Reject a request |
| DELETE | `/v1/friends/{friend_id}` | Remove a friend |
| GET | `/v1/capes/me` | Read the signed-in player's cape status |
| PUT | `/v1/capes/me` | Publish or replace the player's single cape |
| DELETE | `/v1/capes/me` | Unpublish the player's cape |
| GET | `/v1/capes/player/{minecraft_uuid}` | Public PNG texture for a Minecraft player |
| POST | `/v1/client/heartbeat` | Mark the authenticated linked account as using Orange Client |
| GET | `/v1/client/presence?uuids=...` | Return the listed UUIDs with a recent Orange Client heartbeat |

For protected routes send `Authorization: Bearer ACCESS_TOKEN`. Registration and login accept `{"username":"Example","password":"a-long-password"}`.

Cape publishing requires a session created by `POST /v1/auth/minecraft`, which verifies the current Microsoft/Minecraft account. Send the PNG file itself as the raw request body with `Content-Type: image/png`; accepted images must be exactly **64×32 pixels** and no larger than **256 KiB**. A `PUT` replaces that player's one existing cape instead of creating another. The API returns a versioned `texture_url` from `GET /v1/capes/me`; other players can fetch it by Minecraft UUID. Publishing, replacing and downloading capes are free.

The launcher/mod presence endpoints use the Orange bearer token from the verified Microsoft sign-in flow. The client should send a heartbeat about every 30 seconds; the server treats accounts as active for **90 seconds** after the last heartbeat. Presence lookup accepts 1–80 comma-separated Minecraft UUIDs and returns only matching, recently active Orange Client users. This is a lightweight status signal, not a chat or social messaging service.

## Before public deployment

- Use a managed PostgreSQL database and a private database password.
- Serve the API behind HTTPS.
- Set `ORANGE_CORS_ORIGINS` to the exact web origins that need browser access.
- Store secrets in the hosting provider's secret manager, never in Git.
- Configure backups, monitoring and rate limits at the edge.
- Add account recovery and abuse reporting before opening registration to the public.

The development database password in `compose.yaml` is for local use only and must not be reused on a public server.

## Starten ohne Shell (Windows)

1. Installiere Docker Desktop und öffne es. Warte, bis Docker Desktop vollständig gestartet ist.
2. Doppelklicke auf `Start-Orange-Server.vbs`. Das Skript startet die Datenbank und API im Hintergrund und zeigt eine Meldung an. Es öffnet kein Terminalfenster.
3. Prüfe danach `http://localhost:8080/health`. Bei Erfolg wird `{"status":"ok"}` angezeigt.
4. Zum Stoppen doppelklicke auf `Stop-Orange-Server.vbs`. Die PostgreSQL-Daten bleiben erhalten.

Bei einem Fehler findest du die Ausgabe in `orange-server-start.log` bzw. `orange-server-stop.log`. Docker Desktop muss während des Betriebs geöffnet bleiben.
