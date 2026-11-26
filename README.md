# Orange Client

**Orange Client** is a community-maintained, Orange-branded Minecraft Java launcher for Windows, based on the open-source OneLauncher project.

It provides Microsoft account sign-in, Minecraft instance management, modpack installation, Java management, and mod browsing through the services supported by its upstream foundation. The default launcher appearance is **Orange Pixel Dark**, with orange as the accent colour.

> **Project status:** This is an early community fork. It is not an official Polyfrost release, and it does not yet have an independent public release history or an independent Microsoft app registration.

## Downloads and builds

Windows x64 builds are produced from the `orange-client` branch by GitHub Actions:

- [Windows build workflow](https://github.com/u2803045005abc09734-byte/OrangeLauncher3/actions)
- [Repository source](https://github.com/u2803045005abc09734-byte/OrangeLauncher3)

The workflow assembles the pinned upstream source, applies the Orange Client changes and security overrides, runs the Windows account-data protection tests, builds the launcher and installer, and scans the final Windows files with Microsoft Defender.

**Code signing:** SignPath Foundation integration is prepared but requires separate project approval and configuration. Until a signed release is published, downloaded builds may still be reported as an unknown publisher by Windows.

## Microsoft account sign-in

Microsoft authentication currently uses the public OAuth application registration from the upstream OneLauncher project so that Microsoft sign-in continues to work. As a result, Microsoft's consent screen may still display **OneClient/OneLauncher**. Changing the displayed consent-screen name requires an Orange Client-owned Microsoft application registration; changing the launcher UI cannot rename somebody else's Microsoft registration.

## Data migration and account protection

On Windows, saved account data is protected with Windows DPAPI. The first Orange Client launch migrates the existing OneClient settings and data folder to the Orange Client data folder where applicable. Do not manually delete the old launcher data folder before the first Orange Client launch.

## Building from source

The project uses Rust and GitHub Actions. The Windows workflow builds the source assembled by `prepare-source.ps1` and `apply-orange-branding.ps1`.

## Upstream attribution and license

Orange Client is a modified community fork of [Polyfrost/OneLauncher](https://github.com/Polyfrost/OneLauncher), based on its GPL-3.0-licensed source. Original copyright notices, upstream attribution, and the GPL-3.0 license are retained. Internal crate names and compatibility identifiers remain unchanged where renaming them would break dependencies, saved data, or launcher functionality.

Some upstream service endpoints (including metadata and related launcher services) are retained so the corresponding features continue to work. These are third-party services, not Orange Client-owned services.

See [LICENSE](LICENSE) for the full license text.
