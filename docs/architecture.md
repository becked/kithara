# Kithara Architecture

A cross-platform soundboard application for Old World game audio, built with Tauri, Svelte 5, and Rust.

## Overview

Kithara extracts and plays sound effects from Old World's Wwise audio system. It provides a searchable, categorized interface for browsing and playing unit sounds, combat audio, and story event stingers.

## Platform Support

- macOS
- Windows
- Linux

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Frontend | Svelte 5 | Reactive UI |
| Backend | Rust + Tauri 2 | Audio playback, file I/O, IPC |
| Audio Format | OGG Vorbis | Cross-platform, good compression, fast decode |
| Storage | SQLite | Sound catalog and metadata |
| Extraction | vgmstream | Convert Wwise .wem to .ogg |

## Audio Scope

### Included (~1,130 sounds, ~80-120MB cache)

| Category | Count | Source | Description |
|----------|-------|--------|-------------|
| Unit animations | 1,044 | Audio_Animation.bnk | Attacks, deaths, hits, footsteps, vocals |
| UI/Story events | ~73 | Audio_2D.bnk | Calamities, story stingers, tile events |
| Combat gameplay | ~15 | Audio_3D.bnk | Projectile impacts, unit killed |

### Excluded

| Category | Size | Reason |
|----------|------|--------|
| Music | 795MB | Not useful for soundboard |
| Ambience | 301MB | Environmental loops, not interesting |
| UI clicks | - | Generic, boring |

### Supported Units (55 types)

African Elephant, Amazon Cavalry, Archer, Axeman, Ballista, Barbarian Raider, Battering Ram, Bireme, Camel Archer, Caravan, Cataphract, Chariot, Clubthrower, Crossbowman, Disciple, Dromon, Elite variants, Gaesata, Hastatus, Hoplite, Horse Archer, Horseman, Huscarl, Javelineer, Legionary, Light Chariot, Longbowman, Maceman, Mangonel, Militia, Nomad variants, Onager, Peltast, Pikeman, Polybolos, Scout, Settler, Siege Tower, Skirmisher, Slinger, Spearman, Swordsman, Trireme, Warlord, Warrior, Worker

## Architecture

### First Run / Extraction Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     FIRST RUN / EXTRACTION                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Detect Old World install path                               │
│     ├─ macOS: ~/Library/Application Support/Steam/steamapps/    │
│     ├─ Windows: C:\Program Files (x86)\Steam\steamapps\         │
│     ├─ Linux: ~/.steam/steam/steamapps/                         │
│     └─ Fallback: prompt user to locate                          │
│                                                                  │
│  2. Parse Wwise metadata                                        │
│     ├─ Events.xml → event names, IDs, durations, categories     │
│     └─ SoundbanksInfo.xml → file ID mappings                    │
│                                                                  │
│  3. Extract audio from .bnk soundbanks                          │
│     ├─ Parse DIDX section → locate embedded .wem byte offsets   │
│     ├─ Extract .wem bytes to temp directory                     │
│     └─ Convert .wem → .ogg via vgmstream-cli (batch)            │
│                                                                  │
│  4. Build catalog database                                      │
│     └─ SQLite: event_name, category, unit_type, file_path,      │
│                duration, tags                                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Runtime Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          FRONTEND                                │
│                         (Svelte 5)                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Search    │  │  Category   │  │      Sound Grid         │  │
│  │   Input     │  │  Sidebar    │  │  (clickable buttons)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Now Playing Bar                          ││
│  │              [sound name]  ▶ ■  [duration]                  ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Tauri IPC (invoke)
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                          BACKEND                                 │
│                          (Rust)                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  commands.rs                                                     │
│  ├─ search_sounds(query, filters) → Vec<Sound>                  │
│  ├─ get_categories() → Vec<Category>                            │
│  ├─ get_units() → Vec<UnitType>                                 │
│  ├─ play_sound(id) → ()                                         │
│  ├─ stop_sound() → ()                                           │
│  └─ get_extraction_status() → ExtractionStatus                  │
│                                                                  │
│  catalog.rs                                                      │
│  ├─ query sounds from SQLite                                    │
│  └─ full-text search support                                    │
│                                                                  │
│  player.rs                                                       │
│  ├─ rodio-based audio playback                                  │
│  ├─ play/stop/queue management                                  │
│  └─ playback state events → frontend                            │
│                                                                  │
│  extractor.rs (first-run only)                                  │
│  ├─ bnk_parser: read DIDX/DATA sections                         │
│  ├─ wem_extractor: extract embedded audio                       │
│  └─ converter: shell to vgmstream-cli                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## File Structure

```
kithara/
├── docs/
│   ├── architecture.md          # This file
│   └── old-world-audio-reference.md
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs              # Tauri entry point
│   │   ├── commands.rs          # IPC command handlers
│   │   ├── catalog.rs           # SQLite catalog queries
│   │   ├── player.rs            # Audio playback (rodio)
│   │   ├── extractor/
│   │   │   ├── mod.rs
│   │   │   ├── bnk_parser.rs    # Parse Wwise soundbanks
│   │   │   ├── wem_extractor.rs # Extract .wem from .bnk
│   │   │   ├── converter.rs     # .wem → .ogg conversion
│   │   │   └── metadata.rs      # Parse Events.xml
│   │   └── models.rs            # Shared data types
│   └── resources/
│       └── vgmstream-cli        # Platform-specific binaries
├── src/
│   ├── app.html
│   ├── app.css
│   ├── lib/
│   │   ├── components/
│   │   │   ├── SoundGrid.svelte
│   │   │   ├── SoundButton.svelte
│   │   │   ├── Search.svelte
│   │   │   ├── CategorySidebar.svelte
│   │   │   ├── UnitFilter.svelte
│   │   │   ├── NowPlaying.svelte
│   │   │   └── ExtractionProgress.svelte
│   │   ├── stores/
│   │   │   ├── sounds.ts        # Sound catalog state
│   │   │   ├── player.ts        # Playback state
│   │   │   └── filters.ts       # Search/filter state
│   │   └── api.ts               # Tauri invoke wrappers
│   └── routes/
│       └── +page.svelte         # Main app page
├── static/
├── package.json
├── svelte.config.js
├── vite.config.ts
└── README.md
```

## Data Models

### Sound

```typescript
interface Sound {
  id: string;
  eventName: string;        // e.g., "Warrior_Attack_A_cmbt_impact"
  displayName: string;      // e.g., "Warrior Attack Impact"
  category: Category;
  unitType: string | null;  // e.g., "Warrior", null for non-unit sounds
  subcategory: string;      // e.g., "Attack", "Death", "Hit"
  duration: number;         // seconds
  filePath: string;         // relative path in cache
  tags: string[];           // searchable tags
}

type Category =
  | "unit_attack"
  | "unit_death"
  | "unit_hit"
  | "unit_movement"
  | "unit_vocal"
  | "combat"
  | "ui_event"
  | "story_event"
  | "calamity";
```

### Catalog Schema (SQLite)

```sql
CREATE TABLE sounds (
  id TEXT PRIMARY KEY,
  event_name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  category TEXT NOT NULL,
  unit_type TEXT,
  subcategory TEXT,
  duration_ms INTEGER NOT NULL,
  file_path TEXT NOT NULL,
  tags TEXT,  -- JSON array
  created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sounds_category ON sounds(category);
CREATE INDEX idx_sounds_unit_type ON sounds(unit_type);
CREATE VIRTUAL TABLE sounds_fts USING fts5(
  event_name, display_name, tags,
  content='sounds',
  content_rowid='rowid'
);

CREATE TABLE metadata (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- Stores: events_xml_hash, extraction_date, game_version
```

## Cache Location

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/com.kithara.app/` |
| Windows | `%APPDATA%\kithara\` |
| Linux | `~/.local/share/kithara/` |

### Cache Structure

```
kithara/
├── catalog.db           # SQLite database
├── sounds/
│   ├── unit/
│   │   ├── warrior/
│   │   │   ├── attack_a_impact.ogg
│   │   │   ├── death_a_vocal.ogg
│   │   │   └── ...
│   │   ├── archer/
│   │   └── ...
│   ├── combat/
│   ├── ui_event/
│   └── story_event/
└── vgmstream/           # Extracted tool (if not bundled)
```

## Extraction Process Details

### BNK File Structure

Wwise soundbanks use a chunked format:

```
BKHD (Bank Header)
  └─ Version, bank ID

DIDX (Data Index)
  └─ Array of: { file_id: u32, offset: u32, size: u32 }

DATA (Audio Data)
  └─ Concatenated .wem files at indexed offsets

HIRC (Hierarchy)
  └─ Sound objects, event actions (optional parsing)
```

### Extraction Steps

1. **Parse DIDX**: Read file index to get embedded .wem locations
2. **Map IDs to Events**: Cross-reference with Events.xml
3. **Extract .wem bytes**: Read from DATA section at indexed offsets
4. **Convert to OGG**: `vgmstream-cli -o output.ogg input.wem`
5. **Organize files**: Place in category/unit subdirectories
6. **Build catalog**: Insert metadata into SQLite

### Estimated Times

| Step | Duration |
|------|----------|
| Parse metadata | ~1 second |
| Extract from .bnk | ~30 seconds |
| Convert to OGG | ~2-3 minutes |
| Build catalog | ~5 seconds |
| **Total** | **~3-4 minutes** |

Progress shown in UI with percentage and current file.

## Rust Dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
rodio = { version = "0.19", features = ["vorbis"] }
quick-xml = "0.36"
walkdir = "2"
byteorder = "1"          # For parsing binary .bnk files
thiserror = "1"
tokio = { version = "1", features = ["process"] }  # For async vgmstream
directories = "5"        # Cross-platform app directories
```

## Cache Invalidation

On app launch:

1. Check if Old World is still installed at known path
2. Compute hash of `Events.xml`
3. Compare with stored hash in `metadata` table
4. If different, prompt user: "Old World was updated. Re-extract sounds?"

## Future Roadmap

### v1.0 (MVP)
- [x] Architecture design
- [ ] Extract unit animation sounds
- [ ] Basic grid UI with search
- [ ] Category filtering
- [ ] Audio playback

### v1.1
- [ ] Favorites system
- [ ] Keyboard shortcuts (1-9 for quick play)
- [ ] Dark/light theme

### v1.2
- [ ] Custom playlists/soundboards
- [ ] Export sounds (with attribution)
- [ ] Waveform visualization

### v2.0
- [ ] Support other Wwise-based games
- [ ] Plugin system for game-specific extractors
