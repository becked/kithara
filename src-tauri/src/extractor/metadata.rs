//! Wwise metadata XML parsers.
//! Parses soundbank XML files to map WEM file IDs to names and metadata.

use quick_xml::events::Event as XmlEvent;
use quick_xml::Reader;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Parsed WEM file info from soundbank XML IncludedMemoryFiles
#[derive(Debug, Clone)]
pub struct WwiseFileInfo {
    pub id: u32,
    pub short_name: String, // Original source filename
    pub path: String,       // WEM file path
}

/// Parsed streamed file info from SoundbanksInfo.xml
#[derive(Debug, Clone)]
pub struct StreamedFileInfo {
    pub id: u32,
    pub short_name: String,
}

/// Parse SoundbanksInfo.xml to get streamed music file mappings
pub fn parse_soundbanks_info_xml(path: &Path) -> Result<HashMap<u32, StreamedFileInfo>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read SoundbanksInfo.xml: {}", e))?;

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut files = HashMap::new();
    let mut buf = Vec::new();
    let mut in_streamed_files = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Start(e)) if e.name().as_ref() == b"StreamedFiles" => {
                in_streamed_files = true;
            }
            Ok(XmlEvent::End(e)) if e.name().as_ref() == b"StreamedFiles" => {
                in_streamed_files = false;
            }
            Ok(XmlEvent::Start(e)) if in_streamed_files && e.name().as_ref() == b"File" => {
                let mut file_info = StreamedFileInfo {
                    id: 0,
                    short_name: String::new(),
                };

                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"Id" {
                        file_info.id = parse_attr_u32(&attr.value);
                    }
                }

                // Parse child elements for ShortName
                loop {
                    match reader.read_event_into(&mut buf) {
                        Ok(XmlEvent::Start(child)) if child.name().as_ref() == b"ShortName" => {
                            if let Ok(XmlEvent::Text(text)) = reader.read_event_into(&mut buf) {
                                file_info.short_name = String::from_utf8_lossy(&text).to_string();
                            }
                        }
                        Ok(XmlEvent::End(end)) if end.name().as_ref() == b"File" => break,
                        Ok(XmlEvent::Eof) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }

                if file_info.id > 0 && !file_info.short_name.is_empty() {
                    files.insert(file_info.id, file_info);
                }
            }
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(files)
}

/// Discover soundbank pairs by scanning for Audio_*.xml files containing IncludedMemoryFiles.
/// Returns Vec of (xml_filename, bnk_filename) pairs.
pub fn discover_soundbanks(game_dir: &Path) -> Result<Vec<(String, String)>, String> {
    let mut pairs = Vec::new();

    let entries = std::fs::read_dir(game_dir)
        .map_err(|e| format!("Failed to read game directory: {}", e))?;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();

        if !file_name.starts_with("Audio_") || !file_name.ends_with(".xml") {
            continue;
        }

        let bnk_name = file_name.replace(".xml", ".bnk");
        if !game_dir.join(&bnk_name).exists() {
            continue;
        }

        let xml_path = game_dir.join(&file_name);
        let content = std::fs::read_to_string(&xml_path)
            .map_err(|e| format!("Failed to read {}: {}", file_name, e))?;

        if content.contains("IncludedMemoryFiles") {
            pairs.push((file_name, bnk_name));
        }
    }

    pairs.sort();
    Ok(pairs)
}

/// Parse Event ObjectPath attributes from a soundbank XML to extract unit names.
/// Looks for paths like `\Events\Animation\units\Archer\...` and extracts "Archer".
/// Returns a sorted, deduplicated Vec of unit name strings.
pub fn parse_event_unit_names(path: &Path) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read XML for event parsing: {}", e))?;

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut unit_names = HashSet::new();
    let mut buf = Vec::new();
    let mut in_included_events = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Start(e)) if e.name().as_ref() == b"IncludedEvents" => {
                in_included_events = true;
            }
            Ok(XmlEvent::End(e)) if e.name().as_ref() == b"IncludedEvents" => {
                in_included_events = false;
            }
            Ok(XmlEvent::Empty(e))
                if in_included_events && e.name().as_ref() == b"Event" =>
            {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"ObjectPath" {
                        let obj_path = String::from_utf8_lossy(&attr.value).to_string();
                        // Pattern: \Events\Animation\units\{UnitName}\...
                        // Must match lowercase "units" exactly (capitalized \Units\ is shared combat events)
                        let segments: Vec<&str> = obj_path.split('\\').collect();
                        if let Some(units_idx) = segments.iter().position(|s| *s == "units") {
                            if let Some(unit_name) = segments.get(units_idx + 1) {
                                if !unit_name.is_empty() {
                                    unit_names.insert(unit_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    let mut sorted: Vec<String> = unit_names.into_iter().collect();
    sorted.sort();
    Ok(sorted)
}

/// Parse soundbank XML (Audio_Animation.xml, etc.) to get WEM file ID -> metadata mapping
pub fn parse_soundbank_xml(path: &Path) -> Result<HashMap<u32, WwiseFileInfo>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read soundbank XML: {}", e))?;

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut files = HashMap::new();
    let mut buf = Vec::new();
    let mut in_memory_files = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Start(e)) if e.name().as_ref() == b"IncludedMemoryFiles" => {
                in_memory_files = true;
            }
            Ok(XmlEvent::End(e)) if e.name().as_ref() == b"IncludedMemoryFiles" => {
                in_memory_files = false;
            }
            Ok(XmlEvent::Start(e)) | Ok(XmlEvent::Empty(e))
                if in_memory_files && e.name().as_ref() == b"File" =>
            {
                let mut file_info = WwiseFileInfo {
                    id: 0,
                    short_name: String::new(),
                    path: String::new(),
                };

                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"Id" {
                        file_info.id = parse_attr_u32(&attr.value);
                    }
                }

                // Parse child elements for ShortName and Path
                if !e.name().as_ref().is_empty() && e.name().as_ref() == b"File" {
                    loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(XmlEvent::Start(child)) | Ok(XmlEvent::Empty(child)) => {
                                let tag_name = child.name();
                                if tag_name.as_ref() == b"ShortName" {
                                    if let Ok(XmlEvent::Text(text)) = reader.read_event_into(&mut buf)
                                    {
                                        file_info.short_name =
                                            String::from_utf8_lossy(&text).to_string();
                                    }
                                } else if tag_name.as_ref() == b"Path" {
                                    if let Ok(XmlEvent::Text(text)) = reader.read_event_into(&mut buf)
                                    {
                                        file_info.path = String::from_utf8_lossy(&text).to_string();
                                    }
                                }
                            }
                            Ok(XmlEvent::End(end)) if end.name().as_ref() == b"File" => break,
                            Ok(XmlEvent::Eof) => break,
                            Err(_) => break,
                            _ => {}
                        }
                    }
                }

                if file_info.id > 0 && !file_info.short_name.is_empty() {
                    files.insert(file_info.id, file_info);
                }
            }
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(files)
}

fn parse_attr_u32(value: &[u8]) -> u32 {
    String::from_utf8_lossy(value).parse().unwrap_or(0)
}

/// Parse short_name from soundbank XML to extract metadata.
/// `known_units` is derived dynamically from Event ObjectPaths at extraction time.
/// Format: "cmbt.rng.slinger.short.00.MSTR.wav" or "mv.obj.arrowRattle.MSTR.09.wav"
pub fn parse_short_name(short_name: &str, known_units: &[String]) -> (String, Option<String>, String) {
    // Remove file extension
    let name = short_name.trim_end_matches(".wav").trim_end_matches(".WAV");

    // Split by dots
    let parts: Vec<&str> = name.split('.').collect();
    let name_lower = name.to_lowercase();

    // Determine category from prefix
    let category = if name_lower.starts_with("cmbt") {
        "combat"
    } else if name_lower.starts_with("mv") || name_lower.contains("step") || name_lower.contains("hoof") {
        "movement"
    } else if name_lower.starts_with("vcl") || name_lower.contains("grunt") || name_lower.contains("vocal") {
        "vocal"
    } else if name_lower.contains("bodyfall") || name_lower.contains("death") {
        "death"
    } else if name_lower.contains("weapon") || name_lower.contains("arrow") || name_lower.contains("bow") {
        "weapon"
    } else if name_lower.contains("impact") {
        "impact"
    } else if name_lower.starts_with("ui") {
        "ui"
    } else if name_lower.contains("ambience") || name_lower.contains("ambient") {
        "ambience"
    } else {
        "other"
    };

    // Look for unit type
    let mut unit_type: Option<String> = None;
    for known in known_units {
        if name_lower.contains(&known.to_lowercase()) {
            unit_type = Some(known.to_string());
            break;
        }
    }

    // Build subcategory from meaningful parts
    let subcategory = parts
        .iter()
        .filter(|p| {
            !p.is_empty()
                && !p.chars().all(|c| c.is_ascii_digit())
                && p.len() > 2
                && **p != "MSTR"
                && **p != "SFX"
        })
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("_");

    (category.to_string(), unit_type, subcategory)
}

/// Format display name from soundbank short_name
/// Converts "cmbt.rng.slinger.short.00.MSTR.wav" to "Combat Range Slinger"
pub fn format_short_name_display(short_name: &str) -> String {
    // Remove file extension
    let name = short_name.trim_end_matches(".wav").trim_end_matches(".WAV");

    // Noise words to filter out
    let noise_words = ["mstr", "sfx", "a", "b", "c", "00", "01", "02", "03", "04", "05", "06", "07", "08", "09"];

    // Split by dots and underscores
    let parts: Vec<&str> = name
        .split(|c| c == '.' || c == '_')
        .filter(|p| {
            !p.is_empty()
                && !p.chars().all(|c| c.is_ascii_digit())
                && p.len() > 1
                && !noise_words.contains(&p.to_lowercase().as_str())
        })
        .take(5) // Limit to avoid overly long names
        .collect();

    // Expand common abbreviations and capitalize
    parts
        .iter()
        .map(|p| {
            let expanded = match p.to_lowercase().as_str() {
                "cmbt" => "Combat",
                "rng" => "Range",
                "mv" => "Movement",
                "vcl" => "Vocal",
                "obj" => "Object",
                "hrs" => "Horse",
                _ => {
                    // Capitalize first letter
                    let mut chars = p.chars();
                    match chars.next() {
                        None => return String::new(),
                        Some(first) => {
                            return first.to_uppercase().chain(chars).collect();
                        }
                    }
                }
            };
            expanded.to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a music track title from soundbank short_name
/// Converts "mus.theme.title.MSTR.wav" to "Theme Title"
pub fn format_music_title(short_name: &str) -> String {
    // Remove file extension
    let name = short_name.trim_end_matches(".wav").trim_end_matches(".WAV");

    // Noise words to filter out
    let noise_words = ["mus", "bgm", "music", "mstr", "sfx", "a", "b", "c", "loop", "lp"];

    // Split by dots and underscores
    let parts: Vec<&str> = name
        .split(|c| c == '.' || c == '_')
        .filter(|p| {
            !p.is_empty()
                && !p.chars().all(|c| c.is_ascii_digit())
                && p.len() > 1
                && !noise_words.contains(&p.to_lowercase().as_str())
        })
        .collect();

    if parts.is_empty() {
        return short_name.to_string();
    }

    // Capitalize each word
    parts
        .iter()
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format a streamed music file title
/// Handles formats like:
/// - "Shope, Shope.wav"
/// - "44-16 WAVs\06_Christopher Tin_Zealot King (Assyria)_44-16_082321.wav"
/// - "Violin Concerto - II - Philip Glass.wav"
pub fn format_streamed_music_title(short_name: &str) -> String {
    // Remove file extension and path prefix
    let name = short_name
        .trim_end_matches(".wav")
        .trim_end_matches(".WAV");

    // Get just the filename if there's a path
    let name = name.rsplit(['\\', '/']).next().unwrap_or(name);

    // Handle "XX_Artist_Title_XX_XXXXXX" format (Christopher Tin tracks)
    if name.contains("Christopher Tin") || name.contains("_44-16_") {
        // Format: "06_Christopher Tin_Zealot King (Assyria)_44-16_082321"
        let parts: Vec<&str> = name.split('_').collect();
        if parts.len() >= 3 {
            // Skip track number, get artist and title
            let artist_title: Vec<&str> = parts.iter()
                .skip(1) // Skip track number
                .take_while(|p| !p.contains("44-16") && !p.chars().all(|c| c.is_ascii_digit()))
                .cloned()
                .collect();

            if !artist_title.is_empty() {
                return artist_title.join(" - ");
            }
        }
    }

    // Handle suffix patterns like "_C49E5CC0"
    let name = if let Some(idx) = name.rfind("_C49E5CC0") {
        &name[..idx]
    } else {
        name
    };

    // Clean up other noise patterns
    let clean = name
        .replace(".MSTR", "")
        .replace("_MSTR", "")
        .trim()
        .to_string();

    if clean.is_empty() {
        short_name.to_string()
    } else {
        clean
    }
}

