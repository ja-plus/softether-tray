# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/ja-plus/softether-tray/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/ja-plus/softether-tray/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ja-plus/softether-tray/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ja-plus/softether-tray/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ja-plus/softether-tray/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ja-plus/softether-tray/releases/tag/v0.1.0
