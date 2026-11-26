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
if (-not $rootCargoText.Contains('"Orange Client Contributors"')) {
    $authorsHeader = 'authors = ['
    if ($rootCargoText.Contains($authorsHeader)) {
        $authorsInsert = "authors = [" + [Environment]::NewLine + '  "Orange Client Contributors",'
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

# Package and installer properties. The crate and binary IDs stay stable internally to avoid breaking Cargo dependencies.
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'description = "OneClient Freya App"' 'description = "Orange Client Minecraft Launcher"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'product-name = "OneClient"' 'product-name = "Orange Client"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'identifier = "org.polyfrost.OneClient"' 'identifier = "com.orangeclient.launcher"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'publisher = "Polyfrost Inc."' 'publisher = "Orange Client Contributors"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'copyright = "© 2026 Polyfrost Inc."' 'copyright = "© 2026 Polyfrost Inc. and Orange Client contributors"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'homepage = "https://polyfrost.org/projects/oneclient"' 'homepage = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'long-description = "Next-generation open source Minecraft launcher"' 'long-description = "Orange Client is an open-source Minecraft launcher with Microsoft account sign-in, modpack management and an Orange-themed interface."'
Replace-SourceText "packages/oneclient_app/Cargo.toml" 'This build of OneClient needs the Microsoft Visual C++ Redistributable' 'This build of Orange Client needs the Microsoft Visual C++ Redistributable'

# Windows executable metadata.
Replace-SourceText "packages/oneclient_app/build.rs" 'res.set("CompanyName", "Polyfrost Inc.");' 'res.set("CompanyName", "Orange Client Contributors");'
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
Replace-SourceText "packages/oneclient_core/src/settings/launcher.rs" 'theme: "dark".to_string(),' 'theme: "minecraft-dark".to_string()'
Replace-SourceText "packages/oneclient_app/assets/themes/dark.json" '"name": "PolyGlass Dark"' '"name": "Orange Dark"'
Replace-SourceText "packages/oneclient_app/assets/themes/dark.json" '"accent": "#2B4BFF"' '"accent": "#FF7A18"'
Replace-SourceText "packages/oneclient_app/assets/themes/light.json" '"name": "PolyGlass Light"' '"name": "Orange Light"'
Replace-SourceText "packages/oneclient_app/assets/themes/light.json" '"accent": "#2B4BFF"' '"accent": "#E65C00"'
Replace-SourceText "packages/oneclient_app/assets/themes/minecraft-dark.json" '"name": "Minecraft Dark"' '"name": "Orange Pixel Dark"'
Replace-SourceText "packages/oneclient_app/assets/themes/minecraft-dark.json" '"accent": "#2B4BFF"' '"accent": "#FF7A18"'
Replace-SourceText "packages/oneclient_app/assets/themes/minecraft-light.json" '"name": "Minecraft Light"' '"name": "Orange Pixel Light"'
Replace-SourceText "packages/oneclient_app/assets/themes/minecraft-light.json" '"accent": "#2B4BFF"' '"accent": "#E65C00"'

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

# Update the on-screen wordmark and make the standalone logo asset match.
Replace-SourceText "packages/oneclient_app/src/theme/mod.rs" '.text("ONECLIENT")' '.text("ORANGE CLIENT")'
$logo = Get-SourceFile "packages/oneclient_app/assets/logo.svg"
$logoSvg = @'
<svg width="200" height="32" viewBox="0 0 200 32" fill="none" xmlns="http://www.w3.org/2000/svg">
  <text x="0" y="24" fill="currentColor" font-family="Poppins, Arial, sans-serif" font-size="21" font-weight="800" letter-spacing="0.4">ORANGE CLIENT</text>
</svg>
'@
[System.IO.File]::WriteAllText($logo, $logoSvg, $utf8)
Replace-SourceText "packages/oneclient_app/src/components/navbar/app_navbar.rs" '.size(COMPACT_LOGO_PX)
                .color(primary_color(over_art))' '.size(COMPACT_LOGO_PX)
                .color(theme::colors::brand())'

# Replace product-name mentions in UI string literals, but preserve translation keys and source identifiers.
$uiSource = Join-Path $source "packages/oneclient_app/src"
foreach ($file in Get-ChildItem -LiteralPath $uiSource -Recurse -Filter "*.rs" -File) {
    $content = [System.IO.File]::ReadAllText($file.FullName)
    $updated = [regex]::Replace(
        $content,
        '"(?<value>(?:\\.|[^"\\])*)"',
        [System.Text.RegularExpressions.MatchEvaluator]{
            param($match)
            $value = $match.Groups["value"].Value.Replace("OneClient", "Orange Client")
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

# Verify the resulting visible branding before compiling.
$checks = @(
    @{ Path = "packages/oneclient_app/src/constants.rs"; Text = 'WINDOW_APP_ID: &str = "com.orangeclient.launcher"' },
    @{ Path = "packages/oneclient_app/src/constants.rs"; Text = 'WINDOW_TITLE: &str = "Orange Client"' },
    @{ Path = "packages/oneclient_app/Cargo.toml"; Text = 'product-name = "Orange Client"' },
    @{ Path = "packages/oneclient_app/assets/themes/minecraft-dark.json"; Text = '"accent": "#FF7A18"' },
    @{ Path = "packages/oneclient_core/src/settings/launcher.rs"; Text = 'theme: "minecraft-dark".to_string()' }
)
foreach ($check in $checks) {
    $content = [System.IO.File]::ReadAllText((Get-SourceFile $check.Path))
    if (-not $content.Contains($check.Text)) {
        throw "Orange branding verification failed in $($check.Path): missing $($check.Text)"
    }
}

Write-Host "Orange Client branding and default Orange Pixel theme applied successfully."
Write-Host "The Microsoft app registration and upstream service endpoints are intentionally retained so sign-in and launcher services keep working."
