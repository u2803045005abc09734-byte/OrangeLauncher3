use std::path::PathBuf;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

const API_BASE: &str = "https://orange-api-7kkb.onrender.com";
const SESSION_FILE: &str = "orange-session.dat";
const SESSION_MAGIC: &[u8] = b"ORANGE_CLIENT_API_SESSION_DPAPI_V1\n";

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct OrangeUser {
    pub id: Uuid,
    pub username: String,
    pub created_at: String,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct OrangeSession {
    pub access_token: String,
    pub expires_at: String,
    pub user: OrangeUser,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct OrangeFriendRequest {
    pub id: Uuid,
    pub username: String,
    pub created_at: String,
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    expires_at: String,
    user: OrangeUser,
}

#[derive(Deserialize)]
struct ErrorBody {
    error: Option<String>,
}

#[derive(Serialize)]
struct Credentials<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct FriendRequestInput<'a> {
    username: &'a str,
}

#[derive(Deserialize)]
struct SentFriendRequest {
    username: String,
}

fn api_url(path: &str) -> String {
    format!("{API_BASE}{path}")
}

fn session_path() -> Result<PathBuf, String> {
    let directory = oneclient_common::paths::config_dir()
        .map_err(|error| format!("Could not find the Orange Client data folder: {error}"))?;
    Ok(directory.join(SESSION_FILE))
}

async fn decode_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, String> {
    let status = response.status();
    if !status.is_success() {
        let message = response
            .json::<ErrorBody>()
            .await
            .ok()
            .and_then(|body| body.error)
            .unwrap_or_else(|| format!("Orange server returned HTTP {status}"));
        return Err(if status == reqwest::StatusCode::UNAUTHORIZED {
            format!("AUTH: {message}")
        } else {
            message
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|_| "Orange server returned an invalid response.".to_string())
}

async fn check_response(response: reqwest::Response) -> Result<(), String> {
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let message = response
        .json::<ErrorBody>()
        .await
        .ok()
        .and_then(|body| body.error)
        .unwrap_or_else(|| format!("Orange server returned HTTP {status}"));
    Err(if status == reqwest::StatusCode::UNAUTHORIZED {
        format!("AUTH: {message}")
    } else {
        message
    })
}

pub async fn register(username: &str, password: &str) -> Result<OrangeSession, String> {
    let response = HTTP
        .post(api_url("/v1/auth/register"))
        .json(&Credentials { username, password })
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server. It may be waking up; try again shortly.".to_string())?;
    let auth: AuthResponse = decode_response(response).await?;
    Ok(OrangeSession {
        access_token: auth.access_token,
        expires_at: auth.expires_at,
        user: auth.user,
    })
}

pub async fn login(username: &str, password: &str) -> Result<OrangeSession, String> {
    let response = HTTP
        .post(api_url("/v1/auth/login"))
        .json(&Credentials { username, password })
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server. It may be waking up; try again shortly.".to_string())?;
    let auth: AuthResponse = decode_response(response).await?;
    Ok(OrangeSession {
        access_token: auth.access_token,
        expires_at: auth.expires_at,
        user: auth.user,
    })
}

pub async fn me(token: &str) -> Result<OrangeUser, String> {
    let response = HTTP
        .get(api_url("/v1/me"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    decode_response(response).await
}

pub async fn friends(token: &str) -> Result<Vec<OrangeUser>, String> {
    let response = HTTP
        .get(api_url("/v1/friends"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    decode_response(response).await
}

pub async fn incoming_requests(token: &str) -> Result<Vec<OrangeFriendRequest>, String> {
    let response = HTTP
        .get(api_url("/v1/friends/requests"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    decode_response(response).await
}

pub async fn search_users(token: &str, query: &str) -> Result<Vec<OrangeUser>, String> {
    let response = HTTP
        .get(api_url("/v1/users/search"))
        .query(&[("q", query)])
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    decode_response(response).await
}

pub async fn send_friend_request(token: &str, username: &str) -> Result<String, String> {
    let response = HTTP
        .post(api_url("/v1/friends/requests"))
        .bearer_auth(token)
        .json(&FriendRequestInput { username })
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    let request: SentFriendRequest = decode_response(response).await?;
    Ok(request.username)
}

pub async fn accept_request(token: &str, request_id: Uuid) -> Result<(), String> {
    let response = HTTP
        .post(api_url(&format!("/v1/friends/requests/{request_id}/accept")))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    check_response(response).await
}

pub async fn reject_request(token: &str, request_id: Uuid) -> Result<(), String> {
    let response = HTTP
        .post(api_url(&format!("/v1/friends/requests/{request_id}/reject")))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    check_response(response).await
}

pub async fn remove_friend(token: &str, friend_id: Uuid) -> Result<(), String> {
    let response = HTTP
        .delete(api_url(&format!("/v1/friends/{friend_id}")))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    check_response(response).await
}

pub async fn logout(token: &str) -> Result<(), String> {
    let response = HTTP
        .post(api_url("/v1/auth/logout"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not reach the Orange server.".to_string())?;
    check_response(response).await
}

pub async fn load_saved_session() -> Result<Option<OrangeSession>, String> {
    let path = session_path()?;
    let bytes = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("Could not read the saved Orange account.".to_string()),
    };

    let json = decode_session_bytes(&bytes)?;
    let session = serde_json::from_slice(&json)
        .map_err(|_| "The saved Orange account data is damaged.".to_string())?;
    Ok(Some(session))
}

pub async fn save_session(session: &OrangeSession) -> Result<(), String> {
    let path = session_path()?;
    let directory = path
        .parent()
        .ok_or_else(|| "Could not find the Orange Client data folder.".to_string())?;
    tokio::fs::create_dir_all(directory)
        .await
        .map_err(|_| "Could not create the Orange Client data folder.".to_string())?;

    let json = serde_json::to_vec(session)
        .map_err(|_| "Could not prepare the Orange account for saving.".to_string())?;
    let bytes = encode_session_bytes(&json)?;
    let temporary = path.with_extension("dat.tmp");
    tokio::fs::write(&temporary, bytes)
        .await
        .map_err(|_| "Could not save the Orange account.".to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = tokio::fs::set_permissions(
            &temporary,
            std::fs::Permissions::from_mode(0o600),
        )
        .await;
    }

    tokio::fs::rename(&temporary, &path)
        .await
        .map_err(|_| "Could not finish saving the Orange account.".to_string())?;
    Ok(())
}

pub async fn clear_saved_session() {
    if let Ok(path) = session_path() {
        let _ = tokio::fs::remove_file(path).await;
    }
}

#[cfg(windows)]
fn encode_session_bytes(json: &[u8]) -> Result<Vec<u8>, String> {
    let encrypted = windows_dpapi::protect(json)?;
    let mut result = Vec::with_capacity(SESSION_MAGIC.len() + encrypted.len());
    result.extend_from_slice(SESSION_MAGIC);
    result.extend_from_slice(&encrypted);
    Ok(result)
}

#[cfg(not(windows))]
fn encode_session_bytes(json: &[u8]) -> Result<Vec<u8>, String> {
    Ok(json.to_vec())
}

#[cfg(windows)]
fn decode_session_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    match bytes.strip_prefix(SESSION_MAGIC) {
        Some(ciphertext) => windows_dpapi::unprotect(ciphertext),
        None => Ok(bytes.to_vec()),
    }
}

#[cfg(not(windows))]
fn decode_session_bytes(bytes: &[u8]) -> Result<Vec<u8>, String> {
    Ok(bytes.to_vec())
}

#[cfg(windows)]
mod windows_dpapi {
    use std::{ffi::c_void, ptr, slice};

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

    #[repr(C)]
    struct DataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "Crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(
            data_in: *const DataBlob,
            description: *const u16,
            optional_entropy: *const DataBlob,
            reserved: *mut c_void,
            prompt: *const c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;

        fn CryptUnprotectData(
            data_in: *const DataBlob,
            description: *mut *mut u16,
            optional_entropy: *const DataBlob,
            reserved: *mut c_void,
            prompt: *const c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }

    fn take_blob(blob: &mut DataBlob) -> Result<Vec<u8>, String> {
        if blob.pb_data.is_null() {
            return if blob.cb_data == 0 {
                Ok(Vec::new())
            } else {
                Err("Windows returned an empty Orange account buffer.".to_string())
            };
        }
        let bytes = unsafe { slice::from_raw_parts(blob.pb_data, blob.cb_data as usize) }.to_vec();
        unsafe {
            let _ = LocalFree(blob.pb_data.cast());
        }
        blob.pb_data = ptr::null_mut();
        Ok(bytes)
    }

    fn discard_blob(blob: &mut DataBlob) {
        if !blob.pb_data.is_null() {
            unsafe {
                let _ = LocalFree(blob.pb_data.cast());
            }
            blob.pb_data = ptr::null_mut();
        }
    }

    pub(super) fn protect(plain: &[u8]) -> Result<Vec<u8>, String> {
        let length = u32::try_from(plain.len())
            .map_err(|_| "Orange account data is too large for Windows DPAPI.".to_string())?;
        let input = DataBlob {
            cb_data: length,
            pb_data: plain.as_ptr().cast_mut(),
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let description: Vec<u16> = "Orange Client API session\0".encode_utf16().collect();
        let success = unsafe {
            CryptProtectData(
                &input,
                description.as_ptr(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        if success == 0 {
            let error = std::io::Error::last_os_error();
            discard_blob(&mut output);
            return Err(format!("Windows could not protect the Orange account: {error}"));
        }
        take_blob(&mut output)
    }

    pub(super) fn unprotect(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let length = u32::try_from(ciphertext.len())
            .map_err(|_| "Encrypted Orange account data is too large.".to_string())?;
        let input = DataBlob {
            cb_data: length,
            pb_data: ciphertext.as_ptr().cast_mut(),
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let mut description: *mut u16 = ptr::null_mut();
        let success = unsafe {
            CryptUnprotectData(
                &input,
                &mut description,
                ptr::null(),
                ptr::null_mut(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        if !description.is_null() {
            unsafe {
                let _ = LocalFree(description.cast());
            }
        }
        if success == 0 {
            let error = std::io::Error::last_os_error();
            discard_blob(&mut output);
            return Err(format!("Windows could not unlock the Orange account: {error}"));
        }
        take_blob(&mut output)
    }
}
