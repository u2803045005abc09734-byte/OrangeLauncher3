$ErrorActionPreference = "Stop"

$source = Join-Path $PSScriptRoot "source"
if (-not (Test-Path -LiteralPath $source -PathType Container)) {
    throw "Patched source directory is missing: $source"
}
$utf8 = [System.Text.UTF8Encoding]::new($false)

function Get-SourceFile([string]$relativePath) {
    $path = Join-Path $source $relativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Expected source file is missing: $relativePath"
    }
    return $path
}

function Replace-SourceText(
    [string]$relativePath,
    [string]$oldText,
    [string]$newText,
    [int]$minimumMatches = 1
) {
    $path = Get-SourceFile $relativePath
    $content = [System.IO.File]::ReadAllText($path)
    $count = [regex]::Matches($content, [regex]::Escape($oldText)).Count

    if ($count -lt $minimumMatches) {
        Write-Host "Orange branding: $relativePath already changed upstream or by the Orange patch; skipping this exact replacement."
        return
    }

    $content = $content.Replace($oldText, $newText)
    [System.IO.File]::WriteAllText($path, $content, $utf8)
    Write-Host "Orange branding: updated $relativePath ($count replacement(s))."
}

# Public project identity and metadata. Original upstream attribution and GPL license stay intact.
$rootCargoPath = Get-SourceFile "Cargo.toml"
$rootCargoText = [System.IO.File]::ReadAllText($rootCargoPath)
if (-not $rootCargoText.Contains('"CleanOrange"')) {
    $authorsHeader = 'authors = ['
    if ($rootCargoText.Contains($authorsHeader)) {
        $authorsInsert = "authors = [" + [Environment]::NewLine + '  "CleanOrange",'
        $rootCargoText = $rootCargoText.Replace($authorsHeader, $authorsInsert)
        [System.IO.File]::WriteAllText($rootCargoPath, $rootCargoText, $utf8)
    } else {
        Write-Warning "Workspace authors block was not found; upstream attribution remains unchanged."
    }
}
Replace-SourceText "Cargo.toml" 'documentation = "https://github.com/Polyfrost/OneLauncher"' 'documentation = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"'
Replace-SourceText "Cargo.toml" 'homepage = "https://polyfrost.org/projects/oneclient"' 'homepage = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"'
Replace-SourceText "Cargo.toml" 'repository = "https://github.com/Polyfrost/OneLauncher"' 'repository = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"'

# Visible app name, window identity and update destinations (never update an Orange build from OneClient).
Replace-SourceText "packages/oneclient_app/src/constants.rs" 'pub const WINDOW_APP_ID: &str = "oneclient_app";' 'pub const WINDOW_APP_ID: &str = "com.orangeclient.launcher";'
Replace-SourceText "packages/oneclient_app/src/constants.rs" 'pub const WINDOW_TITLE: &str = "OneClient";' 'pub const WINDOW_TITLE: &str = "Orange Client";'
Replace-SourceText "packages/oneclient_app/src/constants.rs" 'https://github.com/Polyfrost/OneLauncher/releases/latest/download/latest.json' 'https://github.com/u2803045005abc09734-byte/OrangeLauncher3/releases/latest/download/latest.json'
Replace-SourceText "packages/oneclient_app/src/constants.rs" 'https://github.com/Polyfrost/OneLauncher/releases/latest' 'https://github.com/u2803045005abc09734-byte/OrangeLauncher3/releases/latest'

# Normalize application identity constants regardless of values introduced by the Orange source patch.
$constantsPath = Get-SourceFile "packages/oneclient_app/src/constants.rs"
$constantsText = [System.IO.File]::ReadAllText($constantsPath)
$appIdPattern = 'pub const WINDOW_APP_ID:\s*&str\s*=\s*"[^"]*";'
$titlePattern = 'pub const WINDOW_TITLE:\s*&str\s*=\s*"[^"]*";'
$appIdMatches = [regex]::Matches($constantsText, $appIdPattern).Count
$titleMatches = [regex]::Matches($constantsText, $titlePattern).Count
if ($appIdMatches -gt 0) {
    $constantsText = [regex]::Replace($constantsText, $appIdPattern, 'pub const WINDOW_APP_ID: &str = "com.orangeclient.launcher";')
}
if ($titleMatches -gt 0) {
    $constantsText = [regex]::Replace($constantsText, $titlePattern, 'pub const WINDOW_TITLE: &str = "Orange Client";')
}
[System.IO.File]::WriteAllText($constantsPath, $constantsText, $utf8)
Write-Host "Orange identity normalization: WINDOW_APP_ID matches=$appIdMatches; WINDOW_TITLE matches=$titleMatches."
if (($appIdMatches -eq 0) -or ($titleMatches -eq 0)) {
    Write-Host "Current constants.rs content:"
    Get-Content -LiteralPath $constantsPath
}

# Package and installer properties. The crate and binary IDs stay stable internally to avoid breaking Cargo dependencies.
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'description = "OneClient Freya App"' 'description = "Orange Client Minecraft Launcher"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'product-name = "OneClient"' 'product-name = "Orange Client"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'identifier = "org.polyfrost.OneClient"' 'identifier = "com.orangeclient.launcher"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'publisher = "Polyfrost Inc."' 'publisher = "CleanOrange"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'copyright = "© 2026 Polyfrost Inc."' 'copyright = "© 2026 Polyfrost Inc. and Orange Client contributors"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'homepage = "https://polyfrost.org/projects/oneclient"' 'homepage = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'long-description = "Next-generation open source Minecraft launcher"' 'long-description = "Orange Client is an open-source Minecraft launcher with Microsoft account sign-in, modpack management and an Orange-themed interface."'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'This build of OneClient needs the Microsoft Visual C++ Redistributable' 'This build of Orange Client needs the Microsoft Visual C++ Redistributable'

# Windows executable metadata.
Replace-SourceText "packages/oneclient_app/build.rs" 'res.set("CompanyName", "Polyfrost Inc.");' 'res.set("CompanyName", "CleanOrange");'
Replace-SourceText "packages/oneclient_app/build.rs" 'res.set("ProductName", "OneClient");' 'res.set("ProductName", "Orange Client");'
Replace-SourceText "packages/oneclient_app/build.rs" 'res.set("FileDescription", "OneClient");' 'res.set("FileDescription", "Orange Client Minecraft Launcher");'
Replace-SourceText "packages/oneclient_app/build.rs" 'res.set("LegalCopyright", "© 2026 Polyfrost Inc.");' 'res.set("LegalCopyright", "© 2026 Polyfrost Inc. and Orange Client contributors");'
Replace-SourceText "packages/oneclient_app/build.rs" 'res.set("OriginalFilename", "oneclient_app.exe");' 'res.set("OriginalFilename", "orange-client.exe");'
Replace-SourceText "packages/oneclient_app/build.rs" 'res.set("InternalName", "oneclient_app");' 'res.set("InternalName", "orange-client");'

# macOS and Linux package display names.
$infoPlistPath = Join-Path $source "packages/oneclient_app/distribution/Info.plist"
if (Test-Path -LiteralPath $infoPlistPath -PathType Leaf) {
    Replace-SourceText "packages/oneclient_app/distribution/Info.plist" '<string>OneClient</string>' '<string>Orange Client</string>'
} else {
    Write-Host "Orange branding: no macOS Info.plist in this Windows source tree; skipping."
}
$linuxTemplatePath = Join-Path $source "packages/oneclient_app/distribution/linux/org.polyfrost.oneclient.template"
if (Test-Path -LiteralPath $linuxTemplatePath -PathType Leaf) {
    Replace-SourceText "packages/oneclient_app/distribution/linux/org.polyfrost.oneclient.template" 'Name=OneClient' 'Name=Orange Client'
    Replace-SourceText "packages/oneclient_app/distribution/linux/org.polyfrost.oneclient.template" 'Comment=Next-generation open source Minecraft launcher' 'Comment=Orange Client open-source Minecraft launcher'
} else {
    Write-Host "Orange branding: no Linux desktop template in this Windows source tree; skipping."
}

# Default appearance: Orange Pixel Dark, with orange accents across all built-in theme families.
Replace-SourceText "packages/oneclient_core/src/settings/launcher.rs" 'theme: "dark".to_string(),' 'theme: "minecraft-dark".to_string(),'
function Set-OrangeTheme([string]$relativePath, [string]$name, [string]$accent) {
    $path = Get-SourceFile $relativePath
    $theme = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    $theme.name = $name
    $theme.colors.accent = $accent
    $json = $theme | ConvertTo-Json -Depth 12
    [System.IO.File]::WriteAllText($path, $json + [Environment]::NewLine, $utf8)
    Write-Host "Orange branding: configured $name with accent $accent."
}
Set-OrangeTheme "packages/oneclient_app/assets/themes/dark.json" "Orange Dark" "#FF7A18"
Set-OrangeTheme "packages/oneclient_app/assets/themes/light.json" "Orange Light" "#E65C00"
Set-OrangeTheme "packages/oneclient_app/assets/themes/minecraft-dark.json" "Orange Pixel Dark" "#FF7A18"
Set-OrangeTheme "packages/oneclient_app/assets/themes/minecraft-light.json" "Orange Pixel Light" "#E65C00"

# Keep the old OneClient folder as the migration source, but use Orange Client for all new user data.
Replace-SourceText "packages/oneclient_common/src/paths.rs" 'const ORGANIZATION: &str = "Polyfrost";' 'const ORGANIZATION: &str = "OrangeClient";'
Replace-SourceText "packages/oneclient_common/src/paths.rs" 'const APPLICATION: &str = "OneClient";' 'const APPLICATION: &str = "Orange Client";'
Replace-SourceText "packages/oneclient_common/src/paths.rs" 'const APPLICATION: &str = "OneClient-dev";' 'const APPLICATION: &str = "Orange Client-dev";'
Replace-SourceText "packages/oneclient_common/src/paths.rs" @'
pub fn legacy_dir() -> Option<PathBuf> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.data_local_dir().to_path_buf())
}
'@ @'
pub fn legacy_dir() -> Option<PathBuf> {
    // Migrate existing OneClient settings, account data and game library into Orange Client.
    {
        let roaming = BaseDirs::new()?
            .data_dir()
            .join("Polyfrost")
            .join("OneClient");
        if roaming.join(SETTINGS_FILE).is_file() {
            return Some(roaming);
        }

        // Older pre-roaming installs may still have the settings under Local AppData.
        ProjectDirs::from(QUALIFIER, "Polyfrost", "OneClient")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
    }
}
'@
Replace-SourceText "packages/oneclient_common/src/paths.rs" @'
if cfg!(debug_assertions) {
                "OneClient-dev"
            } else {
                "OneClient"
            }
'@ @'
if cfg!(debug_assertions) {
                "Orange Client-dev"
            } else {
                "Orange Client"
            }
'@

# Use the white Orange mark next to the OrangeClient wordmark throughout the launcher.
$orangeMarkPath = Join-Path $PSScriptRoot "branding/orange-logo-mark.svg"
if (-not (Test-Path -LiteralPath $orangeMarkPath -PathType Leaf)) {
    throw "Missing vector Orange Client logo mark: $orangeMarkPath"
}
$orangeMark = [System.IO.File]::ReadAllText($orangeMarkPath)
$markBodyMatch = [regex]::Match($orangeMark, '(?s)<svg\b[^>]*>(?<body>.*?)</svg>')
if (-not $markBodyMatch.Success) {
    throw "The Orange Client vector mark is not a valid SVG document."
}
$orangeMarkBody = $markBodyMatch.Groups["body"].Value

# The pixel-icon wordmark branch also gets the white mark, instead of text alone.
$oldPixelWordmark = @'
    if colors::active_theme().pixel_icons {
        return label()
            .text("ONECLIENT")
            .font_family(font())
            .font_weight(FontWeight::BOLD)
            .font_size(text_size(height * 0.83))
            .color(color)
            .into_element();
    }
'@
$newPixelWordmark = @'
    if colors::active_theme().pixel_icons {
        return rect()
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(7.)
            .child(
                SvgViewer::new((
                    "icons/pixel/icon-logo.svg",
                    AppAssets::get_bytes("icons/pixel/icon-logo.svg").unwrap_or_default(),
                ))
                .show_loader(false)
                .width(Size::px(height))
                .height(Size::px(height))
                .color(color),
            )
            .child(
                label()
                    .text("ORANGECLIENT")
                    .font_family(font())
                    .font_weight(FontWeight::BOLD)
                    .font_size(text_size(height * 0.83))
                    .color(color),
            )
            .into_element();
    }
'@
Replace-SourceText "packages/oneclient_app/src/theme/mod.rs" $oldPixelWordmark $newPixelWordmark

# The non-pixel wordmark combines the white logo mark and OrangeClient text in one vector asset.
$logo = Get-SourceFile "packages/oneclient_app/assets/logo.svg"
$logoSvg = @"
<svg width="250" height="32" viewBox="0 0 250 32" fill="none" xmlns="http://www.w3.org/2000/svg">
  <g transform="scale(0.03125)">
$orangeMarkBody
  </g>
  <text x="42" y="23" fill="currentColor" font-family="Poppins, Arial, sans-serif" font-size="20" font-weight="800" letter-spacing="0.3">ORANGECLIENT</text>
</svg>
"@
[System.IO.File]::WriteAllText($logo, $logoSvg, $utf8)

# Replace the old mark in navigation and secondary-window icons with the same white Orange asset.
$logoAssets = @(
    "packages/oneclient_app/assets/icons/icon-logo.svg",
    "packages/oneclient_app/assets/icons/pixel/icon-logo.svg",
    "packages/oneclient_app/icons/OneClient.icon/Assets/oneclient-mark.svg"
)
foreach ($relativePath in $logoAssets) {
    $path = Join-Path $source $relativePath
    if (Test-Path -LiteralPath $path -PathType Leaf) {
        [System.IO.File]::WriteAllText($path, $orangeMark, $utf8)
        Write-Host "Orange branding: replaced legacy logo asset $relativePath."
    }
}

# Keep the transparent white badge separate from the full-color Windows app icon.
$whiteBadgeSource = Join-Path $PSScriptRoot "branding/orange-in-game-mark.png"
if (-not (Test-Path -LiteralPath $whiteBadgeSource -PathType Leaf)) {
    throw "Missing transparent white Orange Client in-game mark: $whiteBadgeSource"
}
$whiteBadgePath = Join-Path $source "packages/oneclient_app/assets/orange-client-mark-white.png"
Copy-Item -LiteralPath $whiteBadgeSource -Destination $whiteBadgePath -Force
Write-Host "Orange branding: embedded the separate transparent white badge asset."

# The launcher-level status/version label lives in oneclient_common, not the app translation files.
Replace-SourceText "packages/oneclient_common/src/constants.rs" '"OneClient v"' '"Orange Client v"'
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" '.size(COMPACT_LOGO_PX)
                .color(theme::colors::brand())' '.size(COMPACT_LOGO_PX)
                .color(primary_color(over_art))'
# Avoid drawing the logo twice when the full wordmark is visible. The wordmark now includes
# the white Orange mark itself; the standalone compact mark is only shown at narrow widths.
$oldNavbarLogo = @'
        .child(
            Icon::new(IconType::IconLogo)
                .size(COMPACT_LOGO_PX)
                .color(primary_color(over_art))
                .into_element(),
        )
        .maybe(show_logo, |rect| {
            rect.child(NavbarLogo { over_art }.into_element())
        })
'@
$newNavbarLogo = @'
        .maybe(show_logo, |rect| {
            rect.child(NavbarLogo { over_art }.into_element())
        })
        .maybe(!show_logo, |rect| {
            rect.child(
                Icon::new(IconType::IconLogo)
                    .size(COMPACT_LOGO_PX)
                    .color(primary_color(over_art))
                    .into_element(),
            )
        })
'@
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" $oldNavbarLogo $newNavbarLogo


# Replace product-name mentions in UI string literals, but preserve translation keys and source identifiers.
$uiSource = Join-Path $source "packages/oneclient_app/src"
foreach ($file in Get-ChildItem -LiteralPath $uiSource -Recurse -Filter "*.rs" -File) {
    $content = [System.IO.File]::ReadAllText($file.FullName)
    $updated = [regex]::Replace(
        $content,
        '"(?<value>(?:\\.|[^"\\])*)"',
        [System.Text.RegularExpressions.MatchEvaluator]{
            param($match)
            $value = $match.Groups["value"].Value.Replace("OneClient", "Orange Client").Replace("OneLauncher", "Orange Client")
            return '"' + $value + '"'
        }
    )
    if ($updated -cne $content) {
        [System.IO.File]::WriteAllText($file.FullName, $updated, $utf8)
    }
}

# Translate the branded name in every locale's displayed values only; translation keys remain stable internally.
$langDir = Join-Path $source "packages/oneclient_app/lang"
foreach ($file in Get-ChildItem -LiteralPath $langDir -Filter "*.json" -File) {
    $content = [System.IO.File]::ReadAllText($file.FullName)
    $updated = [regex]::Replace(
        $content,
        '(?<prefix>:\s*")(?<value>(?:\\.|[^"\\])*)(?<suffix>")',
        [System.Text.RegularExpressions.MatchEvaluator]{
            param($match)
            $value = $match.Groups["value"].Value.Replace("OneClient", "Orange Client")
            return $match.Groups["prefix"].Value + $value + $match.Groups["suffix"].Value
        }
    )
    if ($updated -cne $content) {
        [System.IO.File]::WriteAllText($file.FullName, $updated, $utf8)
    }
}

# Connect Orange Client's social entry point to the Orange-owned API instead of Polyfrost social.
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'oneclient_common.workspace = true' @'
oneclient_common.workspace = true
reqwest.workspace = true
'@
Replace-SourceText "packages/oneclient_app/src/lib.rs" 'mod assets;' @'
mod assets;
mod orange_api;
'@
Replace-SourceText "packages/oneclient_app/src/view/chat/mod.rs" 'mod overlay;' @'
mod overlay;
mod orange_social;
'@
Replace-SourceText "packages/oneclient_app/src/view/chat/overlay.rs" 'use crate::hooks::{use_chat_allowed, use_dispatch, use_social_open, use_social_popped_notice};' 'use crate::hooks::{use_dispatch, use_social_open, use_social_popped_notice};'
Replace-SourceText "packages/oneclient_app/src/view/chat/overlay.rs" '        let allowed = use_chat_allowed();' ''
Replace-SourceText "packages/oneclient_app/src/view/chat/overlay.rs" '        if !allowed || (!open && !notice) {' '        if !open && !notice {'
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" '        use_chat_allowed, use_chat_unread, use_clusters, use_current_account, use_dispatch,' '        use_clusters, use_current_account, use_dispatch,'
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" '        let unread_chats = use_chat_unread();' ''
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" '        let chat_allowed = use_chat_allowed();' ''
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" @'
        let has_account = settled_or_loading(&current_account)
            .is_none_or(|account| account.is_some_and(|account| account.is_microsoft()));
'@ ''
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" @'
            .maybe_child(chat_allowed.then(|| {
                super::navbar_button(foreground)
                    .tooltip(t("oneclient.components.navbar.social"))
                    .enabled(has_account)
                    .child(social_icon(unread_chats))
                    .on_press(toggle_social)
            }))
'@ @'
            .child(
                super::navbar_button(foreground)
                    .tooltip(t("oneclient.components.navbar.social"))
                    .child(social_icon(false))
                    .on_press(toggle_social),
            )
'@

# Orange Client has no Wardrobe/Marketplace: remove the Wardrobe entry from the launcher navigation.
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" @'

        .child(NavLink {
            active: route == Route::Wardrobe {},
            target: NavTarget::Route(Route::Wardrobe {}),
            nav_label: t("oneclient.components.navbar.wardrobe"),
            over_art,
        })
'@ ''

# Discord Rich Presence: use the Orange Client asset key. The application name and its
# uploaded assets come from the Discord application whose ID is set in the
# ORANGE_CLIENT_DISCORD_CLIENT_ID GitHub secret (create it with the name "Orange Client").
Replace-SourceText "packages/oneclient_discord/src/lib.rs" 'const LARGE_IMAGE: &str = "onelauncher_logo_512";' 'const LARGE_IMAGE: &str = "orange_client_logo";'
Replace-SourceText "packages/oneclient_discord/src/lib.rs" '"Playing Minecraft"' '"Playing Minecraft with Orange Client"'
Replace-SourceText "packages/oneclient_discord/src/lib.rs" '.large_text("OneClient")' '.large_text("Orange Client")'
# Normalize the actual Discord website button after the pinned source patch.
# Upstream revisions may change the button label or destination, so match its Rust structure.
$discordPath = Get-SourceFile "packages/oneclient_discord/src/lib.rs"
$discordText = [System.IO.File]::ReadAllText($discordPath)
$discordButtonPattern = 'Button::new\(\s*"[^"]*"\s*,\s*"[^"]*"\s*\)'
$discordButtonMatches = [regex]::Matches($discordText, $discordButtonPattern)
if ($discordButtonMatches.Count -lt 1) {
    throw "Orange branding: no Discord Rich Presence Button::new(...) call was found."
}
$discordButtonReplacement = 'Button::new("Orange Client", "https://github.com/u2803045005abc09734-byte/OrangeLauncher3")'
$discordButtonRegex = [regex]::new($discordButtonPattern)
$discordText = $discordButtonRegex.Replace($discordText, $discordButtonReplacement, 1)
# Also remove any remaining visible Polyfrost destination in the Discord integration.
$discordText = [regex]::Replace($discordText, 'https?://(?:www\.)?polyfrost\.org/?', 'https://github.com/u2803045005abc09734-byte/OrangeLauncher3', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
[System.IO.File]::WriteAllText($discordPath, $discordText, $utf8)
Write-Host "Orange branding: normalized $($discordButtonMatches.Count) Discord Rich Presence button(s)."



# Remove the original OneClient chat pipeline. Orange Friends and Minecraft's own chat stay independent.
Replace-SourceText "packages/oneclient_app/src/events.rs" 'ChatEvent, Event, EventReceiver, GameEvent, LaunchStage, ProgressEvent, Signal,' 'Event, EventReceiver, GameEvent, LaunchStage, ProgressEvent, Signal,'
Replace-SourceText "packages/oneclient_app/src/events.rs" 'use crate::chat::{ChatMessage, ChatToast};' ''
Replace-SourceText "packages/oneclient_app/src/events.rs" '                        PumpSignal::Chat(job) => {
                            let station = self.station;
                            spawn_global(job.run(station));
                        }' '                        PumpSignal::Chat(_) => {}'
Replace-SourceText "packages/oneclient_app/src/events.rs" '        let mut chat_events: Vec<ChatEvent> = Vec::new();
        let mut presence: Vec<(uuid::Uuid, bool)> = Vec::new();
' ''
Replace-SourceText "packages/oneclient_app/src/events.rs" '                Event::Chat(ChatEvent::PresenceChanged { player, online }) => {
                    presence.push((player, online));
                }
                Event::Chat(event) => chat_events.push(event),' '                Event::Chat(_) => {},'
Replace-SourceText "packages/oneclient_app/src/events.rs" @'
        let mut toasts: Vec<ChatToast> = Vec::new();
        let mut presence_lost = false;
        if !chat_events.is_empty() {
            let mut guard = self.station.write_channel(AppChannel::Chat);
            for event in chat_events {
                match event {
                    ChatEvent::MessageReceived {
                        account,
                        group_id,
                        message_id,
                        sender,
                        content,
                        invite_id,
                        invite_status,
                    } => {
                        if guard.chat.owner != Some(account) {
                            continue;
                        }

                        folded.chat_invites |= invite_id.is_some();

                        let visible = crate::view::chat::chat_on_screen(guard.social_open);
                        let tab = guard.social_tab;
                        let received = guard.chat.receive(
                            group_id,
                            ChatMessage {
                                id: message_id,
                                sender,
                                content,
                                sent_at: Some(chrono::Utc::now().fixed_offset()),
                                edited: false,
                                invite: invite_id
                                    .map(|id| crate::chat::MessageInvite::new(id, invite_status)),
                            },
                            visible,
                            tab,
                        );

                        folded.chat_unknown_group |= received.unknown_group;
                        if received.read && !folded.chat_read.contains(&group_id) {
                            folded.chat_read.push(group_id);
                        }
                        if let Some(toast) = received.toast {
                            toasts.retain(|known| known.group_id() != Some(group_id));
                            toasts.push(toast);
                        }
                    }
                    ChatEvent::MessageEdited {
                        group_id,
                        message_id,
                        content,
                        invite_status,
                    } => guard
                        .chat
                        .edit(group_id, message_id, content, invite_status),
                    ChatEvent::MessageDeleted {
                        group_id,
                        message_id,
                    } => guard.chat.remove(group_id, message_id),
                    ChatEvent::GroupRenamed { group_id, name } => guard.chat.rename(group_id, name),
                    ChatEvent::PresenceChanged { .. } => {}
                    ChatEvent::RosterChanged => folded.chat_roster = true,
                    ChatEvent::FriendRequestReceived { account, sender } => {
                        folded.chat_roster = true;
                        if guard.chat.owner == Some(account) && !guard.chat.is_friend(sender) {
                            toasts.push(ChatToast::FriendRequest { sender });
                        }
                    }
                    ChatEvent::ConnectionChanged { connected } => {
                        guard.chat.connected = connected;
                        if !connected {
                            presence.clear();
                            presence_lost = true;
                        }
                        folded.chat_reconnected |= connected;
                        folded.chat_roster |= connected;
                    }
                }
            }
        }

        if presence_lost || !presence.is_empty() {
            let mut guard = self.station.write_channel(AppChannel::ChatPresence);
            if presence_lost {
                guard.chat.clear_presence();
            }
            for (player, online) in presence {
                guard.chat.set_presence(player, online);
            }
        }

'@ ''
Replace-SourceText "packages/oneclient_app/src/events.rs" @'
        if folded.chat_roster {
            let station = self.station;
            spawn_global(async move { crate::hooks::load_roster(station).await });
        }
        if folded.chat_invites {
            let station = self.station;
            spawn_global(async move { crate::hooks::load_invite_games(station).await });
        }
        if folded.chat_unknown_group {
            let station = self.station;
            spawn_global(async move { crate::hooks::load_conversations(station).await });
        }
        if folded.chat_reconnected {
            let station = self.station;
            spawn_global(async move { crate::hooks::resync_chat(station).await });
            spawn_global(async move { crate::hooks::load_self_status(station).await });
        }
        for toast in toasts {
            let actions = self.actions.clone();
            spawn_global(async move { crate::hooks::notify_chat(actions, toast).await });
        }
        for group_id in std::mem::take(&mut folded.chat_read) {
            let station = self.station;
            spawn_global(async move { crate::hooks::mark_chat_read(station, group_id).await });
        }

'@ ''

Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" @'
        let social_dispatch = dispatch.clone();
        let toggle_social = move |_| {
            if crate::view::chat::is_chat_window_open() {
                crate::view::chat::open_chat_window();
            } else {
                social_dispatch.toggle_social();
            }
        };
'@ @'
        let social_dispatch = dispatch.clone();
        let toggle_social = move |_| social_dispatch.toggle_social();
'@

Replace-SourceText "packages/oneclient_app/src/components/toasts.rs" '        use_chat_allowed, use_dispatch, use_link_confirm, use_notifications_snapshot,' '        use_dispatch, use_link_confirm, use_notifications_snapshot,'
Replace-SourceText "packages/oneclient_app/src/components/toasts.rs" '        let chat_allowed = use_chat_allowed();
' ''
Replace-SourceText "packages/oneclient_app/src/components/toasts.rs" '        let right = if social_open && chat_allowed {' '        let right = if social_open {'

Replace-SourceText "packages/oneclient_app/src/hooks/actions.rs" '        crate::view::chat::close_chat_window();
' ''
Replace-SourceText "packages/oneclient_app/src/hooks/actions.rs" @'
    pub fn pop_out_social(&self) {
        {
            let mut guard = self.station.clone().write_channel(AppChannel::Social);
            guard.social_open = false;
            guard.social_popped_notice = true;
        }
        crate::view::chat::open_chat_window();
    }
'@ @'
    pub fn pop_out_social(&self) {
        // Orange Friends stays inside the launcher; no separate OneClient chat window.
        let mut guard = self.station.clone().write_channel(AppChannel::Social);
        guard.social_open = true;
        guard.social_popped_notice = false;
    }
'@
Replace-SourceText "packages/oneclient_app/src/hooks/actions.rs" '    pub fn dock_social(&self) {
        crate::view::chat::close_chat_window();
' '    pub fn dock_social(&self) {'

Replace-SourceText "packages/oneclient_app/src/components/lobby/mod.rs" '    let mut chat_key = use_state(|| None::<(Uuid, Option<i32>)>);
' ''
Replace-SourceText "packages/oneclient_app/src/components/lobby/mod.rs" @'
    use_side_effect({
        let dispatch = dispatch.clone();
        move || {
            let key = state
                .read()
                .party
                .as_ref()
                .map(|party| (party.id, party.chat_group_id));
            if *chat_key.peek() != key {
                chat_key.set(key);
                dispatch.refresh_chat();
            }
        }
    });

'@ ''

# The old detachable chat window cannot be opened, even by a stale key or UI callback.
Replace-SourceText "packages/oneclient_app/src/view/chat/window.rs" @'
pub fn open_chat_window() {
    let opening = match CHAT_WINDOW.claim() {
        Claim::Focus(id) => {
            Platform::get().focus_window(id);
            return;
        }
        Claim::Busy => return,
        Claim::Launch(guard) => guard,
    };

    let platform = Platform::get();
    let launcher = crate::platform::main_window();
    let anchor = platform.post_callback(move |ctx: &mut RendererContext| {
        let window = ctx.windows.get(&launcher?)?.window();
        let monitor = window.current_monitor()?;

        Some(Anchor {
            position: window.outer_position().ok()?,
            size: window.outer_size(),
            scale_factor: window.scale_factor(),
            monitor_position: monitor.position(),
            monitor_size: monitor.size(),
        })
    });

    spawn_global(async move {
        let _opening = opening;
        let beside = anchor.await.ok().flatten().map(beside_launcher);
        platform.launch_window(window_config(beside)).await;
    });
}
'@ 'pub fn open_chat_window() {}'
Replace-SourceText "packages/oneclient_app/src/view/chat/window.rs" 'pub fn is_chat_window_open() -> bool {
    CHAT_WINDOW.is_open()
}' 'pub fn is_chat_window_open() -> bool {
    false
}'
Replace-SourceText "packages/oneclient_app/src/view/chat/window.rs" 'pub fn is_chat_window_focused() -> bool {
    CHAT_WINDOW.is_open() && CHAT_FOCUSED.load(Ordering::Relaxed)
}' 'pub fn is_chat_window_focused() -> bool {
    false
}'
Replace-SourceText "packages/oneclient_app/src/view/chat/window.rs" 'pub fn chat_on_screen(social_open: bool) -> bool {
    crate::chat::chat_visible(
        is_chat_window_focused(),
        social_open,
        MAIN_FOCUSED.load(Ordering::Relaxed),
    )
}' 'pub fn chat_on_screen(_social_open: bool) -> bool {
    false
}'



# Stop converting shared Poly+ websocket traffic into the old OneClient ChatEvent API.
# The same socket carries Party packets, which remain supported; chat packets are ignored.
Replace-SourceText "packages/oneclient_polyplus/src/lib.rs" 'use oneclient_events::EventBus;' 'use oneclient_events::EventBus;'
Replace-SourceText "packages/oneclient_polyplus/src/lib.rs" 'events: EventBus' '_events: EventBus'
Replace-SourceText "packages/oneclient_polyplus/src/session.rs" 'use crate::packets::{ClientboundPacket, ServerboundPacket};' 'use crate::packets::ServerboundPacket;'
Replace-SourceText "packages/oneclient_polyplus/src/lib.rs" '    events: EventBus,
' ''
Replace-SourceText "packages/oneclient_polyplus/src/lib.rs" @'
    pub fn events(&self) -> &EventBus {
        &self.events
    }

'@ ''
Replace-SourceText "packages/oneclient_polyplus/src/lib.rs" '    fn new(auth: Arc<oneclient_auth::AuthService>, events: EventBus) -> Self {' '    fn new(auth: Arc<oneclient_auth::AuthService>) -> Self {'
Replace-SourceText "packages/oneclient_polyplus/src/lib.rs" '            events,
' ''
Replace-SourceText "packages/oneclient_polyplus/src/lib.rs" 'Arc::new(PlusClient::new(auth, events))' 'Arc::new(PlusClient::new(auth))'

Replace-SourceText "packages/oneclient_polyplus/src/session.rs" 'use oneclient_events::{ChatEvent, EventBus};' ''
Replace-SourceText "packages/oneclient_polyplus/src/session.rs" @'
    if connected {
        client
            .events
            .chat(ChatEvent::ConnectionChanged { connected: false });
    }

'@ ''
Replace-SourceText "packages/oneclient_polyplus/src/session.rs" @'
    client
        .events
        .chat(ChatEvent::ConnectionChanged { connected: true });
'@ ''
Replace-SourceText "packages/oneclient_polyplus/src/session.rs" @'
    pump(
        &mut websocket,
        &client.events,
        PING_INTERVAL,
'@ @'
    pump(
        &mut websocket,
        PING_INTERVAL,
'@
Replace-SourceText "packages/oneclient_polyplus/src/session.rs" @'
async fn pump<F, Fut>(
    websocket: &mut reqwest_websocket::WebSocket,
    events: &EventBus,
    ping_interval: Duration,
'@ @'
async fn pump<F, Fut>(
    websocket: &mut reqwest_websocket::WebSocket,
    ping_interval: Duration,
'@
Replace-SourceText "packages/oneclient_polyplus/src/session.rs" '                dispatch(events, account_id, &payload);
' ''


$plusSessionPath = Get-SourceFile "packages/oneclient_polyplus/src/session.rs"
$plusSessionText = [System.IO.File]::ReadAllText($plusSessionPath)
$chatDispatchPattern = '(?s)fn dispatch\(events: &EventBus, account: Uuid, payload: &str\) \{.*?\n\}\n\n(?=pub\(crate\) async fn current_account)'
$chatDispatchMatches = [regex]::Matches($plusSessionText, $chatDispatchPattern)
if ($chatDispatchMatches.Count -gt 0) {
    $plusSessionText = [regex]::Replace($plusSessionText, $chatDispatchPattern, '', 1)
    [System.IO.File]::WriteAllText($plusSessionPath, $plusSessionText, $utf8)
    Write-Host "Orange Client: removed the OneClient Chat websocket packet dispatcher."
}
$plusSessionVerify = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_polyplus/src/session.rs"))
$plusPolyplusVerify = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_polyplus/src/lib.rs"))
if ($plusSessionVerify.Contains("ChatEvent::") -or $plusSessionVerify.Contains("events.chat(") -or $plusSessionVerify.Contains("dispatch(events, account_id")) {
    throw "Orange Client verification failed: Poly+ websocket still dispatches OneClient Chat packets."
}
if (-not $plusSessionVerify.Contains("party::dispatch(&payload)")) {
    throw "Orange Client verification failed: the independent Party websocket dispatcher was removed."
}
if ($plusPolyplusVerify.Contains("events: EventBus") -or $plusPolyplusVerify.Contains("self.events")) {
    throw "Orange Client verification failed: Poly+ client still holds the old chat event bus."
}

# Verify the resulting visible branding before compiling.
$checks = @(
    @{ Path = "packages/oneclient_app/src/constants.rs"; Text = 'WINDOW_APP_ID: &str = "com.orangeclient.launcher"' },
    @{ Path = "packages/oneclient_app/src/constants.rs"; Text = 'WINDOW_TITLE: &str = "Orange Client"' },
    @{ Path = "packages/oneclient_app/src/theme/mod.rs"; Text = '.text("ORANGECLIENT")' },
    @{ Path = "packages/oneclient_app/src/theme/mod.rs"; Text = '"icons/pixel/icon-logo.svg"' },
    @{ Path = "packages/oneclient_app/src/components/navbar/app_navbar.rs"; Text = '.maybe(!show_logo, |rect|' },
    @{ Path = "packages/oneclient_discord/src/lib.rs"; Text = '.large_text("Orange Client")' },
    @{ Path = "packages/oneclient_discord/src/lib.rs"; Text = 'Button::new("Orange Client", "https://github.com/u2803045005abc09734-byte/OrangeLauncher3")' },
    @{ Path = "packages/oneclient_common/src/constants.rs"; Text = '"Orange Client v"' },
    @{ Path = "packages/oneclient_app/Cargo.toml"; Text = "reqwest.workspace = true" },
    @{ Path = "packages/oneclient_app/src/lib.rs"; Text = "mod orange_api;" },
    @{ Path = "packages/oneclient_app/src/view/chat/mod.rs"; Text = "mod orange_social;" },
    @{ Path = "packages/oneclient_app/src/view/chat/panel.rs"; Text = "OrangeSocialPanel.into_element()" },
    @{ Path = "packages/oneclient_app/src/view/chat/overlay.rs"; Text = "if !open && !notice {" },
    @{ Path = "packages/oneclient_app/src/components/navbar/app_navbar.rs"; Text = "social_icon(false)" },
    @{ Path = "packages/oneclient_app/Cargo.toml"; Text = 'product-name = "Orange Client"' },
    @{ Path = "packages/oneclient_app/assets/themes/minecraft-dark.json"; Text = '"accent": "#FF7A18"' },
    @{ Path = "packages/oneclient_core/src/settings/launcher.rs"; Text = 'theme: "minecraft-dark".to_string()' }
)
$navbarText = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_app/src/components/navbar/app_navbar.rs"))
if ($navbarText.Contains("Route::Wardrobe")) {
    throw "Orange branding verification failed: the Wardrobe navigation entry is still present."
}
$discordVerification = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_discord/src/lib.rs"))
if ($discordVerification -match '(?i)https?://(?:www\.)?polyfrost\.org') {
    throw "Orange branding verification failed: a Polyfrost destination remains in Discord Rich Presence."
}
$chatEventsVerification = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_app/src/events.rs"))
if ($chatEventsVerification.Contains("ChatEvent::") -or $chatEventsVerification.Contains("hooks::notify_chat")) {
    throw "Orange Client verification failed: the legacy OneClient chat event pipeline is still active."
}
if (-not $chatEventsVerification.Contains("Event::Chat(_) => {}")) {
    throw "Orange Client verification failed: chat events are not explicitly ignored."
}
$chatWindowVerification = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_app/src/view/chat/window.rs"))
if (-not $chatWindowVerification.Contains("pub fn open_chat_window() {}")) {
    throw "Orange Client verification failed: the legacy detachable chat window can still be opened."
}
$navbarVerification = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_app/src/components/navbar/app_navbar.rs"))
if ($navbarVerification.Contains("open_chat_window()") -or $navbarVerification.Contains("is_chat_window_open()")) {
    throw "Orange Client verification failed: the social navbar still routes into the legacy chat window."
}
$lobbyVerification = [System.IO.File]::ReadAllText((Get-SourceFile "packages/oneclient_app/src/components/lobby/mod.rs"))
if ($lobbyVerification.Contains("dispatch.refresh_chat()")) {
    throw "Orange Client verification failed: the legacy chat refresh is still tied to party updates."
}
foreach ($check in $checks) {
    $content = [System.IO.File]::ReadAllText((Get-SourceFile $check.Path))
    if (-not $content.Contains($check.Text)) {
        throw "Orange branding verification failed in $($check.Path): missing $($check.Text)"
    }
}

Write-Host "Orange Client branding and default Orange Pixel theme applied successfully."
Write-Host "The Microsoft app registration and upstream service endpoints are intentionally retained so sign-in and launcher services keep working."
