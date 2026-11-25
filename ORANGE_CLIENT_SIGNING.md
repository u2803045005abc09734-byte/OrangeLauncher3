# Orange Client — Windows release signing

This fork is based on Polyfrost/OneLauncher (GPL-3.0-only) and applies the Orange Client patch on top of the pinned OneLauncher 2.12.1 baseline.

## SignPath Foundation setup

The workflow is ready to submit two signing requests, but it cannot use Polyfrost's SignPath account or credentials. Orange Client needs its own SignPath Foundation approval and project.

1. Apply at https://signpath.org/ for the Orange Client project. The project must meet the Foundation's current open-source requirements, including an OSI-approved license, publicly documented functionality, and a release in the form that will be signed.
2. In the approved SignPath organization, create project slug orange-client and a release signing policy slug release-signing.
3. Register this repository as the project's source repository and configure GitHub.com as the trusted build system. Permit the orange-client branch for release signing.
4. Create artifact configuration slug orange-client-binary by uploading .signpath/orange-client-binary.artifact-configuration.xml.
5. Create artifact configuration slug orange-client-installer by uploading .signpath/orange-client-installer.artifact-configuration.xml.
6. Create a SignPath CI API token for a user allowed to submit requests to the release policy.
7. In GitHub, open Settings → Secrets and variables → Actions → New repository secret and add:
   - SIGNPATH_API_TOKEN
   - SIGNPATH_ORGANIZATION_ID

Do not put the API token in source files, commits, workflow YAML, or chat.

## Build behavior

The workflow builds from this fork's orange-client branch. It signs the application executable first, then packages and signs the NSIS installer. It checks the Authenticode signature after each signing step and scans the final files with Microsoft Defender.

Until both SignPath secrets are configured, the workflow deliberately produces artifacts marked UNSIGNED. Those files are for testing only and do not solve the Windows unknown-publisher warning.

A valid signature identifies the SignPath Foundation as publisher; it does not claim that Microsoft has independently reviewed every line of Orange Client.
