# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2026-11-07

### Added

- In-app updates: the version in the sidebar grows an arrow when a newer GitHub release is available. Clicking it downloads and verifies the package; installing can be postponed and done later without downloading again.
- Launching the app again while it is already running (tray or launcher icon) now brings up the existing window instead of starting a second instance.

### Changed

- Status polling resolves the virtual adapter's IPv4 address once per refresh instead of running `ip` three times.
- `curl` is now a package dependency, used for update checks and downloads.

### Fixed

- Connecting or disconnecting from the tray while another action is running no longer races over the same client; the second action reports that an operation is in progress.
- A connection whose tunnel comes up without a usable virtual adapter now warns that subnet routes were not installed instead of reporting success.
- Account names that look like command-line options are rejected before reaching `vpncmd`.
- The no-root route check now distinguishes `DEAD` (route present, tunnel down) from `OK`, matching the privileged helper.
- `Tools/install-sudoers.sh` no longer assumes a hard-coded home directory and accepts the `vpnclient` path as an argument.

## [0.5.0] - 2026-11-05

### Added

- Dedicated "Tunnels & routes" page, moved out of Settings, for the subnet list, tunnel MTU, and DHCP cleanup.
- Start/stop service and rescan actions in the "SoftEther client" page header.
- Shared naming dialog for creating virtual adapters, with a free-name suggestion (`softether`, `softether2`, …); it stays open when creation fails.
- App version in the sidebar, above the quit button.

### Changed

- Settings: startup and client-service options are grouped into labeled rows with switches.
- Closing the window minimizes to tray by default and can be configured to quit instead; quitting from the sidebar now asks for confirmation.
- Subnet chips show a count and require a second click before deleting.
- Settings section intros now expand from a hover info icon instead of occupying space under the title.
- "Start the client service when the app starts" and "Stop the client service when the app exits" default to on, so a fresh install follows the app without extra setup.
- Dropdown popovers now stack above modal dialogs.
- Docs updated for the new "Tunnels & routes" page.

### Fixed

- Closing the window with minimize-on-close disabled now runs shutdown cleanup and exits, instead of leaving the client running.
- Removed the redundant "saved automatically" hints under the language and style pickers.

## [0.4.1] - 2026-11-03

### Added

- RPM bundle for Fedora/openSUSE, built alongside the `.deb`; the release workflow smoke-tests the RPM in a Fedora container before uploading.
- The connections page links to the profiles page when no profile is configured.

### Changed

- Release workflow builds the `.deb` on release, attaches it to the GitHub release, and submits it to the Spark Store in auto mode.

### Fixed

- New-connection placeholders no longer use real profile details (account name and hostname) as examples.

## [0.4.0] - 2026-10-28

### Added

- Virtual NIC management: create, delete, enable, and disable adapters from a dedicated page.
- Client page to browse every SoftEther client directory on disk, switch the active one, and see which profiles live where.
- Connect timeout with human-readable failure explanations (permission prompts, missing DHCP lease, network errors).
- Confirmation dialog for destructive actions.
- `claim-config` action in the privileged helper.

### Changed

- Settings autosave and draft handling improvements.
- Overview and profiles layout polish; i18n and style updates.

### Fixed

- Passwords containing non-ASCII or long passwords are now hashed locally and written to `vpn_client.config`, instead of being silently truncated or stripped by SoftEther's interactive password prompt.

## [0.3.0] - 2026-10-26

### Added

- Unified shutdown cleanup: SIGTERM (shutdown/logout) now performs the same cleanup as quitting from the tray, and cleanup is idempotent.
- Info tips: tab subtitles expand on hover instead of occupying permanent space.

### Changed

- Connection page layout rework: session duration on the disconnect button, cumulative traffic on the title row, and a compact disconnected state.
- Tray menu keeps only "Open window" and the account entries.
- Desktop notifications now use `notify-rust`, since `tauri-plugin-notification` is unusable on Linux.

### Fixed

- Disconnect the current account on app exit; the client service is stopped only if the setting says so and the app started it.
- Missing notifications and leftover `vpnclient` processes after shutdown/logout.

## [0.2.0] - 2026-10-26

### Added

- Bilingual interface (Simplified Chinese and English) with system-locale detection on first launch; tray menu, notifications, and command errors are localized.
- Selectable SoftEther client directory: bundled, auto-detected, or custom.
- UI style selected from the distribution on first install.
- macOS interface style alongside Windows 11 (Fluent) and Deepin 25.
- Dedicated profiles page split from the overview.
- Custom number-input steppers with press-and-hold repeat.
- Settings save on change with a status bar (idle/saving/saved/blocked/error).
- English README and bilingual usage docs.

### Changed

- Settings page split into six card components.
- Account, NIC, and status reads are cached; live rates use sysfs link counters to avoid extra `vpncmd` processes.
- Connection-state re-check TTL relaxed from 3 s to 5 s.

### Fixed

- Left-clicking the tray icon did nothing under DDE/KDE; switched to the `ksni` backend.
- Tray icon missing on Deepin because the backend only emitted `IconPixmap`; vendored `tray-icon` to emit `IconName`.

## [0.1.0] - 2026-10-25

### Added

- Tray-resident SoftEther VPN client GUI for Linux (Tauri + Vue 3).
- Read and manage all existing SoftEther client connection profiles; connect and disconnect.
- Status panel with tunnel IP, server, HUB, MTU, cumulative traffic, and live up/down rate charts.
- Network diagnostics: packet loss and latency checks against any target.
- Subnet route push after a successful connection, with a default-route safety net.
- Desktop notifications on connection changes and optional launch at login.
- Windows 11 (Fluent) and Deepin 25 interface styles.
- Bundled SoftEther client binaries, polkit policy, and privileged network helper.

[Unreleased]: https://github.com/ja-plus/softether-tray/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/ja-plus/softether-tray/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/ja-plus/softether-tray/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ja-plus/softether-tray/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ja-plus/softether-tray/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ja-plus/softether-tray/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ja-plus/softether-tray/releases/tag/v0.1.0
