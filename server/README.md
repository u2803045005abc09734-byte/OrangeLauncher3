# Orange Server

The first Orange-owned API backend for OrangeLauncher3. It is intentionally a separate Rust workspace under `server/`, so it does not become a member of the launcher's Cargo workspace.

## Included in this first backend milestone

- Username/password registration and login.
- Argon2id password hashing; plaintext passwords are never stored.
- Opaque bearer sessions whose hashes are stored in PostgreSQL.
- Session expiration and logout/revocation.
- Username prefix search.
- Friend requests, incoming request list, accept/reject, friends list and remove.
- Database migrations run automatically when the API starts.
- Docker Compose for a local PostgreSQL database and API.

This is a starter service, not a production deployment. It does not yet include email/account recovery, Microsoft account linking, chat messages, presence, parties, skins or cosmetics. Those features need their own API design and client integration.

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

For protected routes send `Authorization: Bearer ACCESS_TOKEN`. Registration and login accept `{"username":"Example","password":"a-long-password"}`.

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
