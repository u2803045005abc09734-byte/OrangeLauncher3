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
Replace-SourceText "packages/oneclient_discord/src/lib.rs" '.buttons(vec![Button::new("Website", "https://polyfrost.org/")]);' '.buttons(vec![Button::new("Orange Client", "https://github.com/u2803045005abc09734-byte/OrangeLauncher3")]);'

# Verify the resulting visible branding before compiling.
$checks = @(
    @{ Path = "packages/oneclient_app/src/constants.rs"; Text = 'WINDOW_APP_ID: &str = "com.orangeclient.launcher"' },
    @{ Path = "packages/oneclient_app/src/constants.rs"; Text = 'WINDOW_TITLE: &str = "Orange Client"' },
    @{ Path = "packages/oneclient_app/src/theme/mod.rs"; Text = '.text("ORANGECLIENT")' },
    @{ Path = "packages/oneclient_app/src/theme/mod.rs"; Text = '"icons/pixel/icon-logo.svg"' },
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
foreach ($check in $checks) {
    $content = [System.IO.File]::ReadAllText((Get-SourceFile $check.Path))
    if (-not $content.Contains($check.Text)) {
        throw "Orange branding verification failed in $($check.Path): missing $($check.Text)"
    }
}

Write-Host "Orange Client branding and default Orange Pixel theme applied successfully."
Write-Host "The Microsoft app registration and upstream service endpoints are intentionally retained so sign-in and launcher services keep working."
