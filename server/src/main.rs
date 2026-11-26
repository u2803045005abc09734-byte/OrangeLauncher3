use std::{env, net::SocketAddr};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

const DUMMY_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk";

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    session_ttl_days: i64,
}

#[derive(Clone)]
struct AuthContext {
    user_id: Uuid,
    token_hash: Vec<u8>,
}

#[derive(Debug)]
struct ApiError(StatusCode, &'static str);

impl ApiError {
    fn bad_request(message: &'static str) -> Self {
        Self(StatusCode::BAD_REQUEST, message)
    }

    fn unauthorized() -> Self {
        Self(StatusCode::UNAUTHORIZED, "invalid username or password")
    }

    fn not_found(message: &'static str) -> Self {
        Self(StatusCode::NOT_FOUND, message)
    }

    fn conflict(message: &'static str) -> Self {
        Self(StatusCode::CONFLICT, message)
    }

    fn internal() -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = (self.0, self.1);
        (status, Json(ErrorBody { error: message })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(?error, "database operation failed");
        Self::internal()
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct FriendRequestInput {
    username: String,
}

#[derive(Deserialize)]
struct UserSearch {
    q: String,
}

#[derive(Serialize)]
struct PublicUser {
    id: Uuid,
    username: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
    token_type: &'static str,
    expires_at: DateTime<Utc>,
    user: PublicUser,
}

#[derive(Serialize)]
struct FriendRequestView {
    id: Uuid,
    username: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct SentRequestView {
    id: Uuid,
    username: String,
    status: &'static str,
}

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let database_url = env::var("DATABASE_URL")?;
    let bind_addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
        .parse()?;
    let session_ttl_days = env::var("SESSION_TTL_DAYS")
        .unwrap_or_else(|_| "30".to_owned())
        .parse::<i64>()?
        .clamp(1, 90);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState {
        pool,
        session_ttl_days,
    };

    let protected_routes = Router::new()
        .route("/v1/me", get(me))
        .route("/v1/auth/logout", post(logout))
        .route("/v1/users/search", get(search_users))
        .route("/v1/friends", get(list_friends))
        .route(
            "/v1/friends/requests",
            get(list_incoming_requests).post(send_friend_request),
        )
        .route(
            "/v1/friends/requests/{request_id}/accept",
            post(accept_friend_request),
        )
        .route(
            "/v1/friends/requests/{request_id}/reject",
            post(reject_friend_request),
        )
        .route("/v1/friends/{friend_id}", delete(remove_friend))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let allowed_origins =
        env::var("ORANGE_CORS_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_owned());
    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|origin| origin.trim().parse::<HeaderValue>().ok())
        .collect();
    if origins.is_empty() {
        return Err("ORANGE_CORS_ORIGINS must contain at least one valid origin".into());
    }

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/auth/register", post(register))
        .route("/v1/auth/login", post(login))
        .merge(protected_routes)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!(%bind_addr, "Orange API listening");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(state): State<AppState>) -> ApiResult<Json<StatusResponse>> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(StatusResponse { status: "ok" }))
}

async fn register(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    let username = input.username.trim().to_owned();
    if !valid_username(&username) {
        return Err(ApiError::bad_request(
            "username must be 3-16 characters using letters, numbers or underscores",
        ));
    }
    if !(10..=128).contains(&input.password.as_bytes().len()) {
        return Err(ApiError::bad_request(
            "password must contain between 10 and 128 bytes",
        ));
    }

    let password_hash = hash_password(input.password).await?;
    let normalized = username.to_ascii_lowercase();

    let created = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "INSERT INTO users (id, username, username_normalized, password_hash)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (username_normalized) DO NOTHING
         RETURNING id, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(&username)
    .bind(&normalized)
    .bind(password_hash)
    .fetch_optional(&state.pool)
    .await?;

    let Some((user_id, created_at)) = created else {
        return Err(ApiError::conflict("that username is already taken"));
    };

    let (access_token, expires_at) = create_session(&state, user_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token,
            token_type: "Bearer",
            expires_at,
            user: PublicUser {
                id: user_id,
                username,
                created_at,
            },
        }),
    ))
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<Credentials>,
) -> ApiResult<Json<AuthResponse>> {
    let normalized = input.username.trim().to_ascii_lowercase();
    let user = sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
        "SELECT id, username, password_hash, created_at
         FROM users
         WHERE username_normalized = $1",
    )
    .bind(normalized)
    .fetch_optional(&state.pool)
    .await?;

    let (user_id, username, password_hash, created_at, exists) = match user {
        Some((id, name, hash, created)) => (id, name, hash, created, true),
        None => (
            Uuid::nil(),
            String::new(),
            DUMMY_PASSWORD_HASH.to_owned(),
            Utc::now(),
            false,
        ),
    };

    let password_matches = verify_password(input.password, password_hash).await;
    if !exists || !password_matches {
        return Err(ApiError::unauthorized());
    }

    let (access_token, expires_at) = create_session(&state, user_id).await?;
    Ok(Json(AuthResponse {
        access_token,
        token_type: "Bearer",
        expires_at,
        user: PublicUser {
            id: user_id,
            username,
            created_at,
        },
    }))
}

async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or(ApiError(
            StatusCode::UNAUTHORIZED,
            "a Bearer access token is required",
        ))?;

    let token_hash = token_digest(token);
    let user_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM sessions
         WHERE token_hash = $1 AND expires_at > now()",
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError(
        StatusCode::UNAUTHORIZED,
        "access token is invalid or expired",
    ))?;

    request.extensions_mut().insert(AuthContext {
        user_id,
        token_hash,
    });
    Ok(next.run(request).await)
}

async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> ApiResult<Json<PublicUser>> {
    let user = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        "SELECT id, username, created_at FROM users WHERE id = $1",
    )
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::not_found("user not found"))?;

    Ok(Json(PublicUser {
        id: user.0,
        username: user.1,
        created_at: user.2,
    }))
}

async fn logout(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> ApiResult<StatusCode> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(auth.token_hash)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn search_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<UserSearch>,
) -> ApiResult<Json<Vec<PublicUser>>> {
    let search = query.q.trim().to_ascii_lowercase();
    if !(2..=16).contains(&search.len())
        || !search
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(ApiError::bad_request(
            "search query must be 2-16 letters, numbers or underscores",
        ));
    }

    let users = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        "SELECT id, username, created_at
         FROM users
         WHERE id <> $1 AND position($2 in username_normalized) = 1
         ORDER BY username_normalized
         LIMIT 20",
    )
    .bind(auth.user_id)
    .bind(search)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        users
            .into_iter()
            .map(|(id, username, created_at)| PublicUser {
                id,
                username,
                created_at,
            })
            .collect(),
    ))
}

async fn send_friend_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(input): Json<FriendRequestInput>,
) -> ApiResult<(StatusCode, Json<SentRequestView>)> {
    let username = input.username.trim().to_ascii_lowercase();
    if !valid_username(&username) {
        return Err(ApiError::bad_request("invalid username"));
    }

    let target = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, username FROM users WHERE username_normalized = $1",
    )
    .bind(username)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::not_found("Orange user not found"))?;

    if target.0 == auth.user_id {
        return Err(ApiError::bad_request("you cannot add yourself"));
    }

    let already_friends = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM friendships
            WHERE (user_a = $1 AND user_b = $2)
               OR (user_a = $2 AND user_b = $1)
         )",
    )
    .bind(auth.user_id)
    .bind(target.0)
    .fetch_one(&state.pool)
    .await?;
    if already_friends {
        return Err(ApiError::conflict("you are already friends"));
    }

    let request_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM friend_requests
            WHERE (sender_id = $1 AND recipient_id = $2)
               OR (sender_id = $2 AND recipient_id = $1)
         )",
    )
    .bind(auth.user_id)
    .bind(target.0)
    .fetch_one(&state.pool)
    .await?;
    if request_exists {
        return Err(ApiError::conflict("a friend request already exists"));
    }

    let request_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO friend_requests (id, sender_id, recipient_id)
         VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING
         RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(auth.user_id)
    .bind(target.0)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::conflict("a friend request already exists"))?;

    Ok((
        StatusCode::CREATED,
        Json(SentRequestView {
            id: request_id,
            username: target.1,
            status: "pending",
        }),
    ))
}

async fn list_incoming_requests(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> ApiResult<Json<Vec<FriendRequestView>>> {
    let requests = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        "SELECT request.id, sender.username, request.created_at
         FROM friend_requests AS request
         JOIN users AS sender ON sender.id = request.sender_id
         WHERE request.recipient_id = $1
         ORDER BY request.created_at DESC",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        requests
            .into_iter()
            .map(|(id, username, created_at)| FriendRequestView {
                id,
                username,
                created_at,
            })
            .collect(),
    ))
}

async fn accept_friend_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(request_id): Path<Uuid>,
) -> ApiResult<Json<StatusResponse>> {
    let mut transaction = state.pool.begin().await?;

    let sender_id = sqlx::query_scalar::<_, Uuid>(
        "DELETE FROM friend_requests
         WHERE id = $1 AND recipient_id = $2
         RETURNING sender_id",
    )
    .bind(request_id)
    .bind(auth.user_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::not_found("friend request not found"))?;

    let (user_a, user_b) = ordered_pair(sender_id, auth.user_id);
    sqlx::query(
        "INSERT INTO friendships (user_a, user_b)
         VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(user_a)
    .bind(user_b)
    .execute(&mut *transaction)
    .await?;

    // A simultaneous request in the opposite direction is no longer needed.
    sqlx::query(
        "DELETE FROM friend_requests
         WHERE sender_id = $1 AND recipient_id = $2",
    )
    .bind(auth.user_id)
    .bind(sender_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(Json(StatusResponse { status: "accepted" }))
}

async fn reject_friend_request(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(request_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let result = sqlx::query("DELETE FROM friend_requests WHERE id = $1 AND recipient_id = $2")
        .bind(request_id)
        .bind(auth.user_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("friend request not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn list_friends(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> ApiResult<Json<Vec<PublicUser>>> {
    let friends = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        "SELECT friend.id, friend.username, friend.created_at
         FROM friendships AS friendship
         JOIN users AS friend
           ON friend.id = CASE
                WHEN friendship.user_a = $1 THEN friendship.user_b
                ELSE friendship.user_a
              END
         WHERE friendship.user_a = $1 OR friendship.user_b = $1
         ORDER BY friend.username_normalized",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        friends
            .into_iter()
            .map(|(id, username, created_at)| PublicUser {
                id,
                username,
                created_at,
            })
            .collect(),
    ))
}

async fn remove_friend(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(friend_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    if friend_id == auth.user_id {
        return Err(ApiError::bad_request("invalid friend id"));
    }

    let (user_a, user_b) = ordered_pair(auth.user_id, friend_id);
    let result = sqlx::query("DELETE FROM friendships WHERE user_a = $1 AND user_b = $2")
        .bind(user_a)
        .bind(user_b)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("friendship not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn hash_password(password: String) -> ApiResult<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| ())
    })
    .await
    .map_err(|_| ApiError::internal())?
    .map_err(|_| ApiError::internal())
}

async fn verify_password(password: String, stored_hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        let Ok(parsed_hash) = PasswordHash::new(&stored_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    })
    .await
    .unwrap_or(false)
}

async fn create_session(state: &AppState, user_id: Uuid) -> ApiResult<(String, DateTime<Utc>)> {
    // Two independently generated UUIDs provide a high-entropy opaque bearer token.
    let token = format!("{}.{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = token_digest(&token);
    let expires_at = Utc::now() + Duration::days(state.session_ttl_days);

    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    Ok((token, expires_at))
}

fn token_digest(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

fn ordered_pair(first: Uuid, second: Uuid) -> (Uuid, Uuid) {
    if first < second {
        (first, second)
    } else {
        (second, first)
    }
}

fn valid_username(username: &str) -> bool {
    (3..=16).contains(&username.len())
        && username
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}
