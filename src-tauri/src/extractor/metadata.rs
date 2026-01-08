//! Wwise metadata XML parsers.
//! Parses soundbank XML files to map WEM file IDs to names and metadata.

use quick_xml::events::Event as XmlEvent;
use quick_xml::Reader;
use std::collections::HashMap;
use std::path::Path;

/// Parsed event from Events.xml (not used for file ID mapping)
#[derive(Debug, Clone)]
pub struct WwiseEvent {
    pub id: u32,
    pub name: String,
    pub object_path: String,
    pub duration_min: f64,
    pub duration_max: f64,
    pub duration_type: String, // "OneShot" or "Infinite"
}

/// Parsed WEM file info from soundbank XML IncludedMemoryFiles
#[derive(Debug, Clone)]
pub struct WwiseFileInfo {
    pub id: u32,
    pub short_name: String, // Original source filename
    pub path: String,       // WEM file path
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

/// Parse Events.xml and return map of event ID -> WwiseEvent
pub fn parse_events_xml(path: &Path) -> Result<HashMap<u32, WwiseEvent>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read Events.xml: {}", e))?;

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut events = HashMap::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Empty(e)) if e.name().as_ref() == b"Event" => {
                let mut event = WwiseEvent {
                    id: 0,
                    name: String::new(),
                    object_path: String::new(),
                    duration_min: 0.0,
                    duration_max: 0.0,
                    duration_type: "OneShot".to_string(),
                };

                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"Id" => event.id = parse_attr_u32(&attr.value),
                        b"Name" => event.name = String::from_utf8_lossy(&attr.value).to_string(),
                        b"ObjectPath" => {
                            event.object_path = String::from_utf8_lossy(&attr.value).to_string()
                        }
                        b"DurationMin" => event.duration_min = parse_attr_f64(&attr.value),
                        b"DurationMax" => event.duration_max = parse_attr_f64(&attr.value),
                        b"DurationType" => {
                            event.duration_type = String::from_utf8_lossy(&attr.value).to_string()
                        }
                        _ => {}
                    }
                }

                if event.id > 0 && !event.name.is_empty() {
                    events.insert(event.id, event);
                }
            }
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(events)
}

fn parse_attr_u32(value: &[u8]) -> u32 {
    String::from_utf8_lossy(value).parse().unwrap_or(0)
}

fn parse_attr_f64(value: &[u8]) -> f64 {
    String::from_utf8_lossy(value).parse().unwrap_or(0.0)
}

/// Known unit types in Old World
const KNOWN_UNITS: &[&str] = &[
    "Archer",
    "Axeman",
    "Ballista",
    "Battering",
    "Bireme",
    "Camel",
    "Caravan",
    "Cataphract",
    "Chariot",
    "Clubthrower",
    "Crossbowman",
    "Disciple",
    "Dromon",
    "Elephant",
    "Gaesata",
    "Hastatus",
    "Hoplite",
    "Horse",
    "Horseman",
    "Huscarl",
    "Javelineer",
    "Legionary",
    "Longbowman",
    "Maceman",
    "Mangonel",
    "Militia",
    "Nomad",
    "Onager",
    "Peltast",
    "Pikeman",
    "Polybolos",
    "Raider",
    "Scout",
    "Settler",
    "Siege",
    "Skirmisher",
    "Slinger",
    "Spearman",
    "Swordsman",
    "Trireme",
    "Warlord",
    "Warrior",
    "Worker",
];

/// Parse event name to extract category, unit type, and subcategory
pub fn parse_event_name(name: &str) -> (String, Option<String>, String) {
    let parts: Vec<&str> = name.split('_').collect();

    // Check for Female_ prefix
    let (unit_start, _has_female) = if parts.first() == Some(&"Female") {
        (1, true)
    } else {
        (0, false)
    };

    // Look for unit type in the first few parts
    let mut unit_type: Option<String> = None;
    for i in unit_start..std::cmp::min(unit_start + 2, parts.len()) {
        if let Some(part) = parts.get(i) {
            for known in KNOWN_UNITS {
                if part.eq_ignore_ascii_case(known) {
                    unit_type = Some(known.to_string());
                    break;
                }
            }
            if unit_type.is_some() {
                break;
            }
        }
    }

    // Determine category based on action keywords in the name
    let name_lower = name.to_lowercase();
    let category = if name_lower.contains("attack") || name_lower.contains("cmbt") {
        "unit_attack"
    } else if name_lower.contains("death") || name_lower.contains("bodyfall") {
        "unit_death"
    } else if name_lower.contains("hit") && name_lower.contains("vocal") {
        "unit_hit"
    } else if name_lower.contains("run") || name_lower.contains("step") || name_lower.contains("footstep")
    {
        "unit_movement"
    } else if name_lower.contains("vocal") || name_lower.contains("grunt") {
        "unit_vocal"
    } else if name_lower.contains("calamity") {
        "calamity"
    } else if name_lower.starts_with("ui_") {
        "ui_event"
    } else if name_lower.contains("story") {
        "story_event"
    } else if unit_type.is_some() {
        // Default to combat for unit sounds
        "combat"
    } else {
        "other"
    };

    // Extract subcategory (action type)
    let subcategory = if let Some(action_idx) = parts
        .iter()
        .position(|p| ["Attack", "Death", "Hit", "Run", "Idle"].contains(p))
    {
        parts
            .get(action_idx)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        parts
            .get(unit_start + 1)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    };

    (category.to_string(), unit_type, subcategory)
}

/// Format a display name from an event name
/// Converts "Warrior_Attack_A_cmbt_impact" to "Warrior Attack Impact"
pub fn format_display_name(event_name: &str) -> String {
    // Noise words to filter out
    let noise_words = [
        "a", "b", "c", "cmbt", "random", "loop", "lp", "rnd", "var",
    ];

    // Split by underscore and filter out noise
    let parts: Vec<&str> = event_name
        .split('_')
        .filter(|p| {
            !p.is_empty()
                && !p.chars().all(|c| c.is_ascii_digit())
                && p.len() > 1
                && !noise_words.contains(&p.to_lowercase().as_str())
        })
        .collect();

    // Capitalize first letter of each word
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

/// Parse short_name from soundbank XML to extract metadata
/// Format: "cmbt.rng.slinger.short.00.MSTR.wav" or "mv.obj.arrowRattle.MSTR.09.wav"
pub fn parse_short_name(short_name: &str) -> (String, Option<String>, String) {
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
    for known in KNOWN_UNITS {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_name_warrior_attack() {
        let (category, unit, subcategory) = parse_event_name("Warrior_Attack_A_cmbt_impact");
        assert_eq!(category, "unit_attack");
        assert_eq!(unit, Some("Warrior".to_string()));
        assert_eq!(subcategory, "Attack");
    }

    #[test]
    fn test_parse_event_name_female_scout() {
        let (category, unit, subcategory) =
            parse_event_name("Female_Scout_Death_bodyfall_fromKnees_dirt");
        assert_eq!(category, "unit_death");
        assert_eq!(unit, Some("Scout".to_string()));
        assert_eq!(subcategory, "Death");
    }

    #[test]
    fn test_parse_event_name_ui_event() {
        let (category, unit, _subcategory) = parse_event_name("UI_Calamity_event_start");
        assert_eq!(category, "calamity");
        assert_eq!(unit, None);
    }

    #[test]
    fn test_format_display_name() {
        assert_eq!(
            format_display_name("Warrior_Attack_A_cmbt_impact"),
            "Warrior Attack Impact"
        );
        // Note: case is preserved from original, just first letter capitalized
        assert_eq!(
            format_display_name("Female_Scout_Death_bodyfall_fromKnees_dirt"),
            "Female Scout Death Bodyfall FromKnees Dirt"
        );
    }
}
