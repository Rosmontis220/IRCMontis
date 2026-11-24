# Changelog

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