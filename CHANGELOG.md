# Changelog

All notable changes to Kithara are documented in this file.

## [0.5.0] - 2026-04-01

### Added
- **Sync Library** button for fast incremental updates — skips already-extracted sounds, only processes new content
- Dynamic soundbank discovery — automatically detects new audio banks from game updates and DLC
- Dynamic unit type detection from Wwise event metadata — no hardcoded unit list
- Audio_Ambience soundbank extraction (26 environmental and weather sounds)

### Changed
- Extraction progress bar rebalanced to give music extraction a fair share (50%) when music is included
- "Rebuild" button demoted to secondary action; "Sync" is now the primary library update action
- Embedded music tracks in Audio_Global bank now correctly routed to music player instead of soundboard

### Removed
- Hardcoded exclusion patterns for previously-unreleased DLC content (Huns, Yuezhi, India, etc.) — all content from the EOTI DLC is now extracted

## [0.4.1] - 2026-02-20

### Changed
- New app icon based on kithara photograph
- Fix Dependabot security alerts and harden CI permissions
- Code cleanup: remove dead code and unused duration display

## [0.4.0] - 2026-02-19

### Changed
- New app icon and warm color scheme redesign

## [0.3.1] - 2026-01-22

### Fixed
- Music player auto-advances to next track when current track finishes

## [0.3.0] - 2026-01-21

### Added
- Winamp-style music player for the game soundtrack with playlist panel
- Streamed music extraction from SoundbanksInfo.xml

## [0.2.1] - 2026-01-11

### Added
- Linux build support (deb package)
- Cache rebuild feature for re-extracting audio
- Exclusion filtering for unreleased game content

### Fixed
- macOS: use system Homebrew binaries (vgmstream, ffmpeg) instead of bundling
- macOS: ffmpeg signing and notarization for DMG distribution
- Linux: use system ffmpeg instead of bundling

## [0.2.0] - 2026-01-09

### Added
- Favorites feature — mark sounds with a heart icon
- App icon in header branding

### Fixed
- Player bar not disappearing after audio finishes

### Removed
- Units dropdown filter (replaced by category sidebar and search)

## [0.1.2] - 2026-01-09

### Fixed
- Hide console windows during audio conversion on Windows

## [0.1.1] - 2026-01-09

### Fixed
- Windows audio extraction: bundle vgmstream with required DLLs
- Windows CI: include sidecar binaries for Tauri build check

## [0.1.0] - 2026-01-09

### Added
- Initial release
- Audio extraction pipeline from Wwise soundbanks (BNK parsing, WEM extraction, vgmstream conversion)
- SQLite catalog with FTS5 full-text search
- Sound playback with rodio
- Category sidebar and search/filter UI
- Cross-platform builds (macOS, Windows)
