$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$source = Join-Path $root "source"
$patchDir = Join-Path $root "patch"
$brandingDir = Join-Path $root "branding"
$patchOutput = Join-Path $env:TEMP "orange-client.patch"

$parts = @(Get-ChildItem -LiteralPath $patchDir -Filter "part-*.b64" -File | Sort-Object Name)
if ($parts.Count -ne 12) {
    throw "Expected exactly 12 patch parts, found $($parts.Count)."
}

$encodedPatch = ($parts | ForEach-Object { (Get-Content -LiteralPath $_.FullName -Raw).Trim() }) -join ""
$compressedPatch = [Convert]::FromBase64String($encodedPatch)
$compressedStream = [System.IO.MemoryStream]::new($compressedPatch)
$gzipStream = [System.IO.Compression.GZipStream]::new(
    $compressedStream,
    [System.IO.Compression.CompressionMode]::Decompress
)
$patchStream = [System.IO.MemoryStream]::new()
$gzipStream.CopyTo($patchStream)
$gzipStream.Dispose()
$compressedStream.Dispose()
[System.IO.File]::WriteAllBytes($patchOutput, $patchStream.ToArray())
$patchStream.Dispose()

if (Test-Path -LiteralPath $source) {
    Remove-Item -LiteralPath $source -Recurse -Force
}

git clone --branch "orange-client" "https://github.com/u2803045005abc09734-byte/OrangeLauncher3.git" $source
if ($LASTEXITCODE -ne 0) { throw "Could not clone the Orange Client source repository." }

# Build the exact commit that triggered this GitHub Actions run. This keeps
# the source revision tied to GitHub's origin metadata for trusted signing.
$sourceRevision = $env:GITHUB_SHA
if ([string]::IsNullOrWhiteSpace($sourceRevision)) {
    $sourceRevision = "orange-client"
}
git -C $source checkout --detach $sourceRevision
if ($LASTEXITCODE -ne 0) { throw "Could not check out the exact Orange Client source revision: $sourceRevision" }

$upstreamCommit = (git -C $source rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($upstreamCommit)) {
    throw "Could not verify the OrangeLauncher3 fork source revision."
}
git -C $source merge-base --is-ancestor "1605bf11248f8cc94eacf8ff1f86981abcab8dc8" HEAD
if ($LASTEXITCODE -ne 0) {
    throw "The Orange Client branch no longer descends from the pinned OneLauncher 2.12.1 baseline."
}

# Restore the pinned upstream README inside the temporary source tree. The fork's
# public README is Orange-branded, while the binary patch was generated against upstream.
git -C $source checkout "1605bf11248f8cc94eacf8ff1f86981abcab8dc8" -- README.md
if ($LASTEXITCODE -ne 0) { throw "Could not restore the pinned upstream README before applying the source patch." }

git -C $source apply --binary --whitespace=nowarn $patchOutput
if ($LASTEXITCODE -ne 0) { throw "Orange Client patch did not apply cleanly." }

# Apply project-owned security overrides after the pinned upstream patch.
# These files are the source of truth for Orange Client-specific security fixes.
$securityOverrides = @(
    @{
        Source = "oneclient_auth_store.rs"
        Destination = "packages/oneclient_auth/src/store.rs"
    },
    @{
        Source = "oneclient_auth_error.rs"
        Destination = "packages/oneclient_auth/src/error.rs"
    },
    @{
        Source = "oneclient_constants.rs"
        Destination = "packages/oneclient_common/src/constants.rs"
    },
    @{
        Source = "orange_api.rs"
        Destination = "packages/oneclient_app/src/orange_api.rs"
        AllowCreate = $true
    },
    @{
        Source = "orange_social.rs"
        Destination = "packages/oneclient_app/src/view/chat/orange_social.rs"
        AllowCreate = $true
    },
    @{
        Source = "orange_social_panel.rs"
        Destination = "packages/oneclient_app/src/view/chat/panel.rs"
    }
)

$overrideDir = Join-Path $root "overrides"
foreach ($override in $securityOverrides) {
    $overridePath = Join-Path $overrideDir $override.Source
    $destinationPath = Join-Path $source $override.Destination
    if (-not (Test-Path -LiteralPath $overridePath -PathType Leaf)) {
        throw "Missing Orange Client security override: $overridePath"
    }
    $destinationDirectory = Split-Path -Parent $destinationPath
    if (-not (Test-Path -LiteralPath $destinationDirectory -PathType Container)) {
        throw "Security override target directory does not exist: $destinationDirectory"
    }
    if (-not $override.AllowCreate -and -not (Test-Path -LiteralPath $destinationPath -PathType Leaf)) {
        throw "Security override target does not exist: $destinationPath"
    }
    Copy-Item -LiteralPath $overridePath -Destination $destinationPath -Force
}

Write-Host "Orange Client security overrides applied."

$cargoManifest = Join-Path $source "Cargo.toml"
$cargoText = Get-Content -LiteralPath $cargoManifest -Raw
$metadataAnchor = 'license = "GPL-3.0-only"'
if (-not $cargoText.Contains('homepage = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"')) {
    if (-not $cargoText.Contains($metadataAnchor)) {
        throw "Could not find workspace package metadata anchor in Cargo.toml."
    }
    $metadata = @'
homepage = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"
repository = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3"
documentation = "https://github.com/u2803045005abc09734-byte/OrangeLauncher3#readme"
'@
    $cargoText = $cargoText.Replace($metadataAnchor, $metadataAnchor + [Environment]::NewLine + $metadata.TrimEnd())
    [System.IO.File]::WriteAllText($cargoManifest, $cargoText, [System.Text.UTF8Encoding]::new($false))
}

$iconFile = Join-Path $brandingDir "orange-app-icon.png"
if (-not (Test-Path -LiteralPath $iconFile -PathType Leaf)) {
    throw "Missing full-color Orange Client app icon: $iconFile"
}
$iconPath = Join-Path $source "packages/oneclient_app/icons/icon.png"
Copy-Item -LiteralPath $iconFile -Destination $iconPath -Force

$gitDir = Join-Path $source ".git"
if (Test-Path -LiteralPath $gitDir) {
    Remove-Item -LiteralPath $gitDir -Recurse -Force
}

Write-Host "Orange Client source assembled successfully."
Write-Host "Upstream revision: $upstreamCommit"
Write-Host "Patched source: $source"

# Apply Orange Client product branding after upstream changes and security overrides.
& (Join-Path $root "apply-orange-branding.ps1")
