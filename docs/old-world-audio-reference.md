# Old World Audio System Reference

## Overview

Old World uses **Wwise** (Audiokinetic's audio middleware) for its sound system. Audio assets are compiled into platform-specific soundbanks rather than stored as individual audio files.

## File Locations

### Soundbank Files (Runtime Audio)
```
OldWorld.app/Contents/Resources/Data/StreamingAssets/Audio/GeneratedSoundBanks/Mac/
```

Contents:
- `*.wem` - Individual audio clips (Wwise Encoded Media), named by numeric ID
- `*.bnk` - Soundbank files containing packed audio and event data
- `Events.xml` - Event definitions and mappings
- `Audio_Animation.xml` - Animation-triggered audio events
- `Audio_2D.xml` - 2D audio events (UI, music, etc.)
- `Audio_3D.xml` - 3D positional audio events
- `Audio_Global.xml` - Global audio events
- `SoundbanksInfo.xml` - Complete soundbank metadata
- `Wwise_IDs.h` - C++ header mapping event names to numeric IDs

### Audio Definition XML (Reference Data)
```
Reference/XML/Infos/audio.xml
```

This XML file defines the audio asset types used by the game's data system, mapping logical audio names to Wwise event names.

## Platform-Specific Builds

The `Mac` folder name indicates platform-specific audio. Wwise generates separate builds per platform due to:

- **Codec differences** - Each platform has optimal audio formats (Vorbis for PC/Mac, XMA for Xbox, ATRAC for PlayStation)
- **Endianness** - Big-endian vs little-endian architectures
- **Quality/compression tradeoffs** - Mobile platforms may use higher compression

Other platforms would have folders like `Windows`, `PS4`, `PS5`, `XboxOne`, etc.

## File Formats

### .wem (Wwise Encoded Media)
Individual audio clips encoded in Wwise's container format. Files are named by numeric ID (e.g., `619052630.wem`), not by descriptive names. The actual codec inside varies by platform (typically Vorbis on PC/Mac).

### .bnk (Soundbank)
Binary files containing:
- Packed audio data
- Event definitions
- Sound structures
- RTPC (Real-Time Parameter Control) data
- State and switch information

## Event System

Wwise uses an event-based system. Game code triggers events by name or ID, and Wwise handles playback.

### Event Structure (from Events.xml)
```xml
<Event
    Id="1941418338"
    Name="Female_Scout_Death_bodyfall_fromKnees_dirt"
    ObjectPath="\Events\Animation\units\Scout\Scout_Death\Female_Scout_Death_bodyfall_fromKnees_dirt"
    GUID="{D1C82ADF-DA56-4885-98BD-FC0801C98DC3}"
    MaxAttenuation="175.000000"
    DurationType="OneShot"
    DurationMin="0.656854"
    DurationMax="0.931000"/>
```

Fields:
- `Id` - Numeric identifier used at runtime
- `Name` - Human-readable event name
- `ObjectPath` - Wwise project hierarchy path
- `GUID` - Unique identifier
- `MaxAttenuation` - Maximum distance for 3D sounds (in game units)
- `DurationType` - `OneShot` or `Infinite` (looping)
- `DurationMin/Max` - Audio duration range in seconds

### Reference XML Structure (audio.xml)
```xml
<Entry>
    <zType>AUDIO_Female_Scout_Death_bodyfall_fromKnees_dirt</zType>
    <zAsset>Female_Scout_Death_bodyfall_fromKnees_dirt</zAsset>
</Entry>
```

The `zType` is the game's internal reference, `zAsset` maps to the Wwise event name.

## Audio Categories

Based on the XML structure, audio is organized into categories:

### Unit Audio
Located under `\Events\Animation\units\{UnitType}\`

Common event types per unit:
- `{Unit}_Attack_A` - Attack sounds
- `{Unit}_Death` - Death sounds (bodyfall, impact, vocal)
- `{Unit}_Hit` - Taking damage sounds
- `{Unit}_Run_A` - Movement/footstep sounds
- `{Unit}_Idle` - Idle animation sounds

### Gender Variants
Some units have female variants with separate audio:
- `Female_Scout_*` - Female scout vocals and foley
- `Female_Worker_*` - Female worker vocals and foley
- `Female_Archer_*` - Female archer vocals and foley

### Example: Female Scout Audio Events
| Event Name | ID | Duration | Description |
|------------|-----|----------|-------------|
| `Female_Scout_Death_bodyfall_fromKnees_dirt` | 1941418338 | 0.66-0.93s | Body hitting ground |
| `Female_Scout_Death_impact_body_fall_chest_dirt_grass` | 2892903672 | 0.75-0.80s | Impact foley |
| `Female_Scout_Death_vocal_grunt_hit_short_Female_B` | 4223293823 | 0.23-0.40s | Death vocal |
| `Female_Scout_Hit_vocal_grunt_hit_short_Female_B` | 1150687608 | 0.23-0.40s | Pain vocal |
| `Female_Scout_Run_A_step_quick_impactful_random` | 2162037791 | 0.13-0.26s | Footsteps |

### Example: Female Worker Audio Events
| Event Name | ID | Duration | Description |
|------------|-----|----------|-------------|
| `Female_Worker_Death_A_vocal_human_pain_death_grunt_Female_A` | 647755627 | 0.65s | Death vocal |
| `Female_Worker_Death_B_bodyfall_hipChest_dirt` | 2138215125 | 0.84s | Body hitting ground |
| `Female_Worker_Hit_A_vocal_human_pain_grunt_Female_A` | 2284937115 | 0.25-0.50s | Pain vocal |
| `Female_Worker_Run_A_step_quick_impactful_random` | 188677683 | 0.13-0.26s | Footsteps |

## Extracting Audio

The `.wem` files can be converted to standard formats using third-party tools:

### Tools
- **vgmstream** - Open-source library/tools for game audio formats
- **Wwise Audio Unpacker** - Extracts and converts Wwise audio
- **Ravioli Game Tools** - GUI tool supporting Wwise formats
- **wwiser** - Python tool for parsing Wwise banks

### Process
1. Locate the `.wem` file by ID (use Events.xml to find the ID for a named event)
2. Use vgmstream or similar to convert `.wem` to `.wav`
3. For packed `.bnk` files, use a bank extractor first

### Example using vgmstream CLI
```bash
# Convert a single .wem file
vgmstream-cli -o output.wav 619052630.wem

# Batch convert all .wem files
for f in *.wem; do vgmstream-cli -o "${f%.wem}.wav" "$f"; done
```

## ID Mapping

To find the audio file for a specific event:

1. Look up the event name in `Reference/XML/Infos/audio.xml` to get the `zAsset`
2. Search for that asset name in `Events.xml` or `SoundbanksInfo.xml`
3. Note the `Id` field
4. The corresponding `.wem` file may be named with that ID, or packed in a `.bnk`

Note: Not all events map directly to `.wem` files. Some events trigger sound structures that combine multiple audio files, apply effects, or use random containers.

## Wwise_IDs.h

This C++ header file provides compile-time constants for event IDs:

```cpp
namespace AK
{
    namespace EVENTS
    {
        static const AkUniqueID FEMALE_SCOUT_DEATH_BODYFALL_FROMKNEES_DIRT = 1941418338U;
        // ... more event IDs
    }
}
```

This allows game code to reference events by name while using numeric IDs at runtime for performance.

## Sound Switches and States

Wwise supports dynamic audio through switches and states. For example, footstep sounds might switch based on:
- Terrain type (dirt, stone, grass, water)
- Movement speed (walk, run)
- Unit weight (light, heavy)

These are defined in the `SwitchContainers` sections of the XML files.

## Further Resources

- [Wwise Documentation](https://www.audiokinetic.com/library/edge/)
- [vgmstream GitHub](https://github.com/vgmstream/vgmstream)
- [Wwise Audio Extraction Guide](https://github.com/bnnm/wwiser)
