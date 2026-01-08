# Kithara Architecture

A cross-platform soundboard application for Old World game audio, built with Tauri 2, Svelte 5, and Rust.

## Overview

Kithara extracts and plays sound effects from Old World's Wwise audio system. It provides a searchable, categorized interface for browsing and playing unit sounds, combat audio, and story event stingers.

## Platform Support

- macOS (ARM64 and Intel)
- Windows
- Linux

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Frontend | Svelte 5 + SvelteKit | Reactive UI with runes |
| Backend | Rust + Tauri 2 | Audio playback, file I/O, IPC |
| Audio Format | OGG Vorbis | Cross-platform, good compression, fast decode |
| Storage | SQLite + FTS5 | Sound catalog with full-text search |
| Conversion | vgmstream-cli → ffmpeg | Two-step WEM → WAV → OGG pipeline |

## Audio Scope

### Included (~1,277 sounds)

| Category | Count | Source | Description |
|----------|-------|--------|-------------|
| Unit animations | ~896 | Audio_Animation.bnk | Attacks, deaths, hits, footsteps, vocals |
| UI/Story events | ~271 | Audio_2D.bnk | Calamities, story stingers, tile events |
| Combat gameplay | ~110 | Audio_3D.bnk | Projectile impacts, environmental |

### Excluded

| Category | Size | Reason |
|----------|------|--------|
| Music | 795MB | Not useful for soundboard |
| Ambience | 301MB | Environmental loops |
| UI clicks | - | Generic sounds |

### Supported Units

Archer, Axeman, Ballista, Battering Ram, Bireme, Camel, Caravan, Cataphract, Chariot, Clubthrower, Crossbowman, Disciple, Dromon, Elephant, Gaesata, Hastatus, Hoplite, Horse, Horseman, Huscarl, Javelineer, Legionary, Longbowman, Maceman, Mangonel, Militia, Nomad, Onager, Peltast, Pikeman, Polybolos, Raider, Scout, Settler, Siege, Skirmisher, Slinger, Spearman, Swordsman, Trireme, Warlord, Warrior, Worker

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
│  2. Parse soundbank XML files for WEM file ID → metadata        │
│     ├─ Audio_Animation.xml → IncludedMemoryFiles section        │
│     ├─ Audio_2D.xml → file IDs and short names                  │
│     └─ Audio_3D.xml → maps file ID to source filename           │
│                                                                  │
│  3. Extract audio from .bnk soundbanks                          │
│     ├─ Parse DIDX section → locate embedded .wem byte offsets   │
│     ├─ Extract .wem bytes to temp directory                     │
│     └─ Convert via sidecar: vgmstream-cli → ffmpeg → .ogg       │
│                                                                  │
│  4. Build catalog database                                      │
│     └─ SQLite: event_name, category, unit_type, file_path,      │
│                display_name, tags (FTS5 indexed)                 │
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
│  ┌──────────────┐  ┌────────────────────────────────────────┐   │
│  │ Unit Filter  │  │           Now Playing Bar              │   │
│  │  (dropdown)  │  │        [sound name]  ▶ ■               │   │
│  └──────────────┘  └────────────────────────────────────────┘   │
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
│  ├─ search_sounds(query, category, unit_type) → Vec<Sound>      │
│  ├─ get_categories() → Vec<Category>                            │
│  ├─ get_unit_types() → Vec<UnitType>                            │
│  ├─ play_sound(id) → ()                                         │
│  ├─ stop_sound() → ()                                           │
│  ├─ start_extraction(game_path) → ()                            │
│  ├─ get_extraction_status() → ExtractionStatus                  │
│  ├─ cancel_extraction() → ()                                    │
│  └─ detect_game_path() → Option<String>                         │
│                                                                  │
│  catalog.rs                                                      │
│  ├─ SQLite with FTS5 full-text search                           │
│  ├─ insert_sound(), search_sounds(), get_categories()           │
│  └─ count_sounds() for extraction detection                     │
│                                                                  │
│  player.rs                                                       │
│  ├─ rodio-based audio playback                                  │
│  ├─ play/stop with Sink management                              │
│  └─ playback state via Mutex<PlayerState>                       │
│                                                                  │
│  extractor/                                                      │
│  ├─ mod.rs: ExtractionManager, run_extraction orchestrator      │
│  ├─ bnk_parser.rs: parse DIDX/DATA, extract WEM bytes           │
│  ├─ metadata.rs: parse soundbank XMLs, categorize sounds        │
│  └─ converter.rs: WEM → WAV → OGG via sidecars                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## File Structure

```
kithara/
├── docs/
│   ├── architecture.md              # This file
│   └── old-world-audio-reference.md # Wwise format details
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json             # Permissions including shell:allow-execute
│   ├── binaries/                    # Sidecar binaries (platform-specific)
│   │   ├── vgmstream-cli-aarch64-apple-darwin
│   │   ├── ffmpeg-aarch64-apple-darwin
│   │   └── ...
│   └── src/
│       ├── main.rs                  # Tauri entry point
│       ├── lib.rs                   # App setup, managed state
│       ├── commands.rs              # IPC command handlers
│       ├── catalog.rs               # SQLite catalog with FTS5
│       ├── player.rs                # Audio playback (rodio)
│       ├── models.rs                # Shared data types + ts-rs bindings
│       └── extractor/
│           ├── mod.rs               # ExtractionManager, run_extraction
│           ├── bnk_parser.rs        # Parse Wwise soundbanks
│           ├── converter.rs         # WEM → OGG conversion
│           └── metadata.rs          # Parse soundbank XMLs
├── src/
│   ├── app.html
│   ├── app.css                      # CSS custom properties (dark theme)
│   ├── lib/
│   │   ├── api.ts                   # Tauri invoke wrappers
│   │   ├── types/                   # Generated from Rust via ts-rs
│   │   │   ├── Sound.ts
│   │   │   ├── Category.ts
│   │   │   └── ...
│   │   ├── stores/
│   │   │   └── sounds.svelte.ts     # Svelte 5 runes state
│   │   └── components/
│   │       ├── SoundGrid.svelte
│   │       ├── SoundButton.svelte
│   │       ├── Search.svelte
│   │       ├── CategorySidebar.svelte
│   │       ├── UnitFilter.svelte
│   │       ├── NowPlaying.svelte
│   │       └── ExtractionProgress.svelte
│   └── routes/
│       └── +page.svelte             # Main app page
├── package.json
├── svelte.config.js
├── vite.config.ts
└── CLAUDE.md                        # AI assistant instructions
```

## Data Models

### Sound

```typescript
interface Sound {
  id: string;
  eventName: string;        // e.g., "cmbt.rng.slinger.short.00.MSTR.wav"
  displayName: string;      // e.g., "Combat Range Slinger"
  category: string;         // e.g., "combat", "movement", "vocal"
  unitType: string | null;  // e.g., "Slinger", null for non-unit sounds
  subcategory: string;      // e.g., "cmbt_rng_slinger"
  duration: number;         // seconds (0 if not available)
  filePath: string;         // absolute path to OGG file
  tags: string[];           // searchable tags
}
```

### Categories

- `combat` - Attack impacts, weapon sounds
- `movement` - Footsteps, hooves, equipment jingle
- `vocal` - Grunts, death cries, horse neighs
- `death` - Bodyfalls, bone cracks
- `weapon` - Arrow shots, bow draws
- `impact` - Hit reactions
- `ui` - UI feedback sounds
- `ambience` - Environmental audio
- `other` - Uncategorized

### Catalog Schema (SQLite)

```sql
CREATE TABLE sounds (
  id TEXT PRIMARY KEY,
  event_name TEXT NOT NULL,
  display_name TEXT NOT NULL,
  category TEXT NOT NULL,
  unit_type TEXT,
  subcategory TEXT,
  duration REAL NOT NULL DEFAULT 0,
  file_path TEXT NOT NULL,
  tags TEXT  -- JSON array
);

CREATE INDEX idx_sounds_category ON sounds(category);
CREATE INDEX idx_sounds_unit_type ON sounds(unit_type);

-- Full-text search
CREATE VIRTUAL TABLE sounds_fts USING fts5(
  event_name, display_name, tags,
  content='sounds',
  content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER sounds_ai AFTER INSERT ON sounds BEGIN
  INSERT INTO sounds_fts(rowid, event_name, display_name, tags)
  VALUES (NEW.rowid, NEW.event_name, NEW.display_name, NEW.tags);
END;
```

## Cache Location

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/com.kithara.app/` |
| Windows | `%APPDATA%\kithara\` |
| Linux | `~/.local/share/kithara/` |

### Cache Structure

```
com.kithara.app/
├── catalog.db               # SQLite database
├── sounds/
│   ├── combat/
│   │   ├── slinger/
│   │   │   └── 8190660_cmbt.rng.slinger.short.00.MSTR.wav.ogg
│   │   └── ...
│   ├── movement/
│   ├── vocal/
│   ├── death/
│   ├── weapon/
│   └── other/
└── temp/                    # Cleaned after extraction
```

## Extraction Process

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
  └─ Sound objects, event actions (skipped)
```

### Soundbank XML Structure

Each BNK has a corresponding XML file with metadata:

```xml
<SoundBank>
  <IncludedMemoryFiles>
    <File Id="8190660" Language="SFX">
      <ShortName>cmbt.rng.slinger.short.00.MSTR.wav</ShortName>
      <Path>SFX\cmbt.rng.slinger.short.00.MSTR_10C4C929.wem</Path>
    </File>
    ...
  </IncludedMemoryFiles>
</SoundBank>
```

### Extraction Pipeline

1. **Parse soundbank XMLs**: Build map of WEM file ID → metadata (short name, path)
2. **Parse BNK DIDX**: Get file ID, offset, and size for each embedded WEM
3. **Extract WEM bytes**: Read from DATA section at indexed offsets to temp file
4. **Convert WEM → WAV**: `vgmstream-cli -o temp.wav temp.wem`
5. **Convert WAV → OGG**: `ffmpeg -i temp.wav -c:a libvorbis -q:a 4 output.ogg`
6. **Categorize**: Parse short_name to determine category and unit type
7. **Insert to catalog**: Add metadata to SQLite with FTS indexing
8. **Cleanup**: Remove temp files

### Conversion Sidecars

Bundled as Tauri external binaries (sidecars):

| Binary | Purpose | Output |
|--------|---------|--------|
| vgmstream-cli | Decode Wwise WEM format | WAV |
| ffmpeg | Encode to OGG Vorbis | OGG (quality 4) |

Two-step conversion is required because vgmstream-cli cannot output OGG directly.

## Rust Dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-shell = "2"          # Sidecar execution
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
rodio = { version = "0.19", features = ["vorbis"] }
quick-xml = "0.37"                # Parse soundbank XMLs
byteorder = "1"                   # Parse binary BNK files
directories = "6"                 # Cross-platform app directories
dirs = "6"                        # Home directory detection
tokio = { version = "1", features = ["sync"] }

[dev-dependencies]
ts-rs = "10"                      # Generate TypeScript types
```

## State Management

### Frontend (Svelte 5 Runes)

```typescript
// sounds.svelte.ts
export const soundsState = $state({
  sounds: [] as Sound[],
  categories: [] as Category[],
  unitTypes: [] as UnitType[],
  loading: false,
  error: null as string | null
});

export const filterState = $state({
  query: '',
  category: null as string | null,
  unitType: null as string | null
});

export const playerState = $state({
  currentSound: null as Sound | null,
  isPlaying: false
});
```

### Backend (Tauri Managed State)

```rust
// Shared state via Arc + Mutex
.manage(catalog)                           // Arc<Catalog>
.manage(player)                            // Arc<Mutex<Player>>
.manage(Arc::new(ExtractionManager::new())) // Thread-safe extraction state
```

## Future Roadmap

### v1.1 - Quality of Life
- [ ] Duration metadata from vgmstream output
- [ ] Keyboard shortcuts (Space play/pause, arrow keys navigate)
- [ ] Volume control (global slider)
- [ ] Sound preview on hover

### v1.2 - User Features
- [ ] Favorites system (star sounds, filter by favorites)
- [ ] Custom playlists/soundboards
- [ ] Re-extraction menu option (for game updates)
- [ ] Search history

### v1.3 - Distribution
- [ ] Static binary builds for vgmstream/ffmpeg (remove Homebrew dependency)
- [ ] Windows and Linux platform testing
- [ ] Auto-updater integration
- [ ] App signing and notarization

### v2.0 - Advanced
- [ ] Waveform visualization
- [ ] Export sounds with attribution
- [ ] Support other Wwise-based games
- [ ] Plugin system for game-specific extractors
