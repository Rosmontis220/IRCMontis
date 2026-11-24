# Changelog

---

## v0.1.1
**Released on:** 2026-11-24

**Title:** Connection and follow-system theme fixes, minimalist restyle and a portable Windows exe

### Fixed
- IRC engine no longer stalls at "connecting": the TLS crypto provider is pinned explicitly so rustls does not panic when Cargo feature unification enables both `ring` and `aws-lc-rs`.
- "Follow system" theme now tracks the OS: switching from dark back to system resets the native window theme instead of staying dark.

### Changed
- Minimalist techwear restyle: frameless window with a custom title bar, flat matte surfaces, hairline borders and a low-saturation olive accent.
- Removed the Fluent web components, the connection/theme/clock toolbar row and the mock status bar; settings now open from a gear icon in the title bar.
- Do-not-disturb is a slide toggle scoped to the current conversation; message search and mute live in the right sidebar alongside the member list.

### Added
- Portable Windows exe that runs without installation, published alongside the MSI and NSIS installers.

---

## v0.1.0
**Released on:** 2026-11-24

**Title:** First Windows release with Libera chat, tray background mode and themes

### Added
- Rust-native IRC engine with TLS and SASL PLAIN authentication against Libera.Chat.
- QQ-style chat UI: session list, message bubbles, identicons, mIRC colors, /me actions and clickable links.
- Light, dark and system themes plus custom background image with adjustable opacity.
- Tray background mode with unread blink, mention badges and per-session mute.
- SQLite message history with in-session search and jump-to-result.
- Emoji panel, nick Tab completion, Enter to send and Shift+Enter for newline.
- Chinese and English interface locales.
- Windows NSIS and MSI installers; Android project configuration prepared without APK builds.

### Fixed
- First message in a newly opened session is no longer dropped (Svelte 5 state proxy copy).
- Tray hide and show now sync window focus so notifications and unread state stay correct.
- Settings dialog no longer fails to mount during Fluent dropdown initialization.