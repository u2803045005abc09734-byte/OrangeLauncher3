pub const DUMMY_REPLACE_NEWLINE: &str = "\n";

// Public client ID used by the upstream OneLauncher Microsoft authentication registration.
// This is not a secret. Set ORANGE_CLIENT_MICROSOFT_CLIENT_ID at build time to use an
// Orange Client-owned Microsoft app registration instead.
const UPSTREAM_MICROSOFT_CLIENT_ID: &str = "9419b7ee-1448-4d1b-b52a-550d8f36ab56";

pub const MICROSOFT_CLIENT_ID: &str = match option_env!("ORANGE_CLIENT_MICROSOFT_CLIENT_ID") {
    Some(id) if !id.is_empty() => id,
    _ => UPSTREAM_MICROSOFT_CLIENT_ID,
};
pub const MINECRAFT_SCOPES: &str = "XboxLive.SignIn XboxLive.offline_access";
// Never embed a private CurseForge API key in a distributed desktop application.
// Requests requiring private credentials must go through a protected backend.
pub const CURSEFORGE_API_KEY: &str = "";
pub const DISCORD_CLIENT_ID: &str = match option_env!("ORANGE_CLIENT_DISCORD_CLIENT_ID") {
    Some(id) => id,
    None => "",
};

// Orange Client's own launcher bundle (mod jar + Fabric API + performance mods) is published as a
// rolling GitHub release by the mod repository. The launcher installs this instead of Polyfrost's
// OneClient bundles, so the closed-source PolyPlus/OneConfig mods are no longer installed.
pub const ORANGE_BUNDLES_URL: &str =
    "https://github.com/u2803045005abc09734-byte/orange-client-2.02/releases/download/bundle-latest";

pub const MODRINTH_API_URL: &str = "https://api.modrinth.com";
pub const MODRINTH_CDN_PREFIX: &str = "https://cdn.modrinth.com/data/";
pub const CURSEFORGE_API_URL: &str = "https://api.curseforge.com/v1";
pub const CURSEFORGE_GAME_ID: u32 = 432;
pub const METADATA_API_URL: &str = "https://meta.polyfrost.org";
pub const MCLOGS_API_URL: &str = "https://api.mclo.gs/1";
pub const SKYCLIENT_BASE_URL: &str =
    "https://raw.githubusercontent.com/SkyblockClient/SkyblockClient-REPO/refs/heads/main/v1";
pub const META_URL_BASE: &str = "https://data-v2.polyfrost.org";
pub const TOS_URL: &str = "https://polyfrost.org/legal/terms";
pub const PRIVACY_URL: &str = "https://polyfrost.org/legal/privacy";
pub const PLUS_BACKEND_URL: &str = "https://plus.polyfrost.org";
pub const CRASH_DATA_URL: &str =
    "https://raw.githubusercontent.com/Polyfrost/CrashData/refs/heads/main/crashes.json";

pub const DISCORD_URL: &str = "https://www.minecraft.net/en-us/community";

pub const SENTRY_DSN: &str = match option_env!("ORANGE_CLIENT_SENTRY_DSN") {
    Some(dsn) => dsn,
    None => "",
};

pub const LAUNCHER_LABEL: &str = concat!(
    "Orange Client v",
    env!("CARGO_PKG_VERSION"),
    cfg_select! {
        debug_assertions => " (debug)",
        _ => " (release)"
    }
);

pub const TARGET_OS: &str = cfg_select! {
    target_os = "windows" => "windows",
    target_os = "macos" => "osx",
    target_os = "linux" => "linux"
};

pub const NATIVE_ARCH: &str = cfg_select! {
    target_arch = "x86" => "32",
    target_arch = "x86_64" => "64",
    _ => "64"
};

pub const JAVA_BIN: &str = if cfg!(windows) { "javaw.exe" } else { "java" };

pub const ARCH_WIDTH: &str = if cfg!(target_pointer_width = "64") {
    "64"
} else {
    "32"
};

pub const LINE_ENDING: &str = if cfg!(windows) { "\r\n" } else { "\n" };
pub const CLASSPATH_SEPARATOR: &str = if cfg!(windows) { ";" } else { ":" };
