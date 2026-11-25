use std::collections::HashMap;

use uuid::Uuid;

#[cfg(windows)]
const AUTH_FILE_MAGIC: &[u8] = b"ORANGE_CLIENT_AUTH_DPAPI_V1\n";

#[cfg(windows)]
mod dpapi {
    use std::ffi::c_void;
    use std::ptr;
    use std::slice;

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
                Err("Windows returned an empty credential buffer.".to_string())
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
            .map_err(|_| "Account data is too large for Windows DPAPI.".to_string())?;
        let input = DataBlob {
            cb_data: length,
            pb_data: plain.as_ptr().cast_mut(),
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let description: Vec<u16> = "Orange Client account data\0".encode_utf16().collect();

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
            return Err(format!("Windows DPAPI encryption failed: {error}"));
        }

        take_blob(&mut output)
    }

    pub(super) fn unprotect(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let length = u32::try_from(ciphertext.len())
            .map_err(|_| "Encrypted account data is too large for Windows DPAPI.".to_string())?;
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
            return Err(format!("Windows DPAPI decryption failed: {error}"));
        }

        take_blob(&mut output)
    }
}

#[cfg(windows)]
fn encrypt_auth_file(plain: &[u8]) -> Result<Vec<u8>, String> {
    let encrypted = dpapi::protect(plain)?;
    let mut result = Vec::with_capacity(AUTH_FILE_MAGIC.len() + encrypted.len());
    result.extend_from_slice(AUTH_FILE_MAGIC);
    result.extend_from_slice(&encrypted);
    Ok(result)
}

#[cfg(windows)]
fn decrypt_auth_file(file: &[u8]) -> Result<(Vec<u8>, bool), String> {
    match file.strip_prefix(AUTH_FILE_MAGIC) {
        Some(ciphertext) => Ok((dpapi::unprotect(ciphertext)?, false)),
        None => Ok((file.to_vec(), true)),
    }
}

use oneclient_common::paths;
use oneclient_events::EventBus;

use crate::data::{AccountKind, MinecraftAccount};
use crate::error::{AuthError, AuthResult};
use crate::offline::{offline_account, validate_offline_username};

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct CredentialsStore {
    pub users: HashMap<Uuid, MinecraftAccount>,
    pub default_user: Option<Uuid>,
}

impl CredentialsStore {
    #[tracing::instrument(level = "debug")]
    pub async fn load() -> AuthResult<Self> {
        let path = paths::auth_file()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let file = match polyio::read(&path).await {
            Ok(file) => file,
            Err(err) => {
                tracing::warn!("failed to read auth file: {err}");
                return Ok(Self::default());
            }
        };

        #[cfg(windows)]
        let (json, needs_encryption) = decrypt_auth_file(&file)
            .map_err(AuthError::CredentialProtection)?;

        #[cfg(not(windows))]
        let json = file;

        let store: Self = match serde_json::from_slice(&json) {
            Ok(store) => store,
            Err(err) => {
                tracing::warn!("failed to parse auth file: {err}");
                return Ok(Self::default());
            }
        };

        // Existing Orange Client installs have a plaintext auth.json. Migrate it
        // immediately on Windows before the account data is used again.
        #[cfg(windows)]
        if needs_encryption {
            tracing::info!("migrating legacy account data to Windows DPAPI protection");
            store.save().await?;
        }

        Ok(store)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn save(&self) -> AuthResult<()> {
        let path = paths::auth_file()?;
        let json = serde_json::to_vec(self).map_err(|err| {
            AuthError::CredentialProtection(format!("could not serialize account data: {err}"))
        })?;

        #[cfg(windows)]
        let bytes = encrypt_auth_file(&json).map_err(AuthError::CredentialProtection)?;

        #[cfg(not(windows))]
        let bytes = json;

        polyio::write_atomic(&path, bytes).await?;
        Ok(())
    }

    pub fn has_microsoft_account(&self) -> bool {
        self.users
            .values()
            .any(|account| account.kind == AccountKind::Microsoft)
    }

    pub fn list_accounts(&self) -> Vec<MinecraftAccount> {
        self.users.values().cloned().collect()
    }

    pub fn get_account(&self, id: Uuid) -> Option<&MinecraftAccount> {
        self.users.get(&id)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(username = %account.username))]
    pub async fn commit_account(
        &mut self,
        account: MinecraftAccount,
        events: &EventBus,
    ) -> AuthResult<MinecraftAccount> {
        self.users.insert(account.id, account.clone());

        if self.default_user.is_none() {
            self.default_user = Some(account.id);
        }

        self.save().await?;
        tracing::debug!("committed account to credentials store");
        events
            .notify("Account added")
            .body(format!("Signed in as {}", account.username))
            .send();
        Ok(account)
    }

    pub fn add_offline_account(&mut self, username: String) -> AuthResult<MinecraftAccount> {
        let account = self.insert_offline_account(username)?;
        Ok(account)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(username = %username))]
    pub async fn add_offline_account_and_save(
        &mut self,
        username: String,
    ) -> AuthResult<MinecraftAccount> {
        let account = self.insert_offline_account(username)?;
        self.save().await?;
        Ok(account)
    }

    fn insert_offline_account(&mut self, username: String) -> AuthResult<MinecraftAccount> {
        if !self.has_microsoft_account() {
            return Err(AuthError::OfflineRequiresMicrosoft);
        }

        validate_offline_username(&username)?;

        if self
            .users
            .values()
            .any(|u| u.username.eq_ignore_ascii_case(&username))
        {
            return Err(AuthError::DuplicateUsername { username });
        }

        let account = offline_account(username);
        self.users.insert(account.id, account.clone());
        Ok(account)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(%account.id))]
    pub async fn commit_refreshed_account(&mut self, account: MinecraftAccount) -> AuthResult<()> {
        self.users.insert(account.id, account);
        self.save().await?;
        tracing::debug!("stored refreshed Microsoft account");
        Ok(())
    }

    /// Keeps the entry and its default slot so the UI can offer a sign-in for
    /// this exact account The refresh token is kept too Microsoft sometimes
    /// rejects a good one and the next renewal retries it
    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn mark_signed_out(&mut self, id: Uuid) -> AuthResult<()> {
        let Some(account) = self.users.get_mut(&id) else {
            return Err(AuthError::AccountNotFound(id));
        };

        account.signed_out = true;
        account.access_token.clear();
        self.save().await?;
        Ok(())
    }

    /// Debug aid swaps in a token Microsoft will reject and backdates expiry so
    /// the next launch walks the real `invalid_grant` path
    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn corrupt_refresh_token(&mut self, id: Uuid) -> AuthResult<()> {
        let Some(account) = self.users.get_mut(&id) else {
            return Err(AuthError::AccountNotFound(id));
        };

        account.refresh_token = "oneclient-debug-invalid-refresh-token".to_string();
        account.expires = chrono::DateTime::UNIX_EPOCH;
        self.save().await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self), fields(%id))]
    pub async fn remove_account(&mut self, id: Uuid) -> AuthResult<Option<MinecraftAccount>> {
        let removed = self.users.remove(&id);

        if self.default_user == Some(id) {
            self.default_user = self.users.keys().copied().next();
        }

        self.save().await?;
        Ok(removed)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(?id))]
    pub async fn set_default_user(&mut self, id: Option<Uuid>) -> AuthResult<()> {
        if let Some(id) = id
            && !self.users.contains_key(&id)
        {
            return Err(AuthError::AccountNotFound(id));
        }

        self.default_user = id;
        self.save().await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn resolve_default_id(&mut self) -> AuthResult<Option<Uuid>> {
        let id = self
            .default_user
            .or_else(|| self.users.keys().copied().next());

        let Some(id) = id else {
            return Ok(None);
        };

        if !self.users.contains_key(&id) {
            return Ok(None);
        }

        if self.default_user != Some(id) {
            self.default_user = Some(id);
            self.save().await?;
        }

        Ok(Some(id))
    }

    /// Never touches the network callers that need a usable token go through
    /// [`crate::AuthService::account_for_launch`]
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn default_account(&mut self) -> AuthResult<Option<MinecraftAccount>> {
        let Some(id) = self.resolve_default_id().await? else {
            return Ok(None);
        };
        let Some(account) = self.users.get(&id).cloned() else {
            return Ok(None);
        };

        if account.is_offline() && !self.has_microsoft_account() {
            return Err(AuthError::OfflineRequiresMicrosoft);
        }

        Ok(Some(account))
    }
}

/// A transient failure must keep the existing token discarding it because
/// Wi-Fi dropped would sign the user out of a working account
pub(crate) fn is_transient_auth_error(err: &AuthError) -> bool {
    matches!(
        err,
        AuthError::Minecraft(crate::error::MinecraftAuthError::RequestError { source, .. })
            if source.is_connect() || source.is_timeout()
    )
}

#[cfg(all(test, windows))]
mod dpapi_tests {
    use super::{AUTH_FILE_MAGIC, decrypt_auth_file, encrypt_auth_file};

    #[test]
    fn account_file_round_trips_through_windows_dpapi() {
        let plain = br#"{"users":{},"default_user":null}"#;
        let encrypted = encrypt_auth_file(plain).expect("DPAPI encryption should succeed");

        assert!(encrypted.starts_with(AUTH_FILE_MAGIC));
        assert_ne!(encrypted.as_slice(), plain.as_slice());

        let (decrypted, needs_encryption) =
            decrypt_auth_file(&encrypted).expect("DPAPI decryption should succeed");
        assert!(!needs_encryption);
        assert_eq!(decrypted.as_slice(), plain.as_slice());
    }

    #[test]
    fn legacy_plaintext_account_file_is_detected_for_migration() {
        let legacy = br#"{"users":{},"default_user":null}"#;
        let (plain, needs_encryption) =
            decrypt_auth_file(legacy).expect("legacy JSON should remain readable");

        assert!(needs_encryption);
        assert_eq!(plain.as_slice(), legacy.as_slice());
    }
}

