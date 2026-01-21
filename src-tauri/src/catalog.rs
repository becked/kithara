//! SQLite catalog module for sound metadata storage and search.
//!
//! Uses rusqlite with FTS5 for full-text search capabilities.

use crate::models::{Category, MusicTrack, Sound, UnitType};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

/// Database connection wrapper for Tauri managed state.
/// Wraps Connection in Mutex since rusqlite Connection is not Sync.
pub struct Catalog {
    conn: Mutex<Connection>,
}

impl Catalog {
    /// Opens or creates the catalog database at the given path.
    /// Creates tables and indexes on first run.
    pub fn open(db_path: PathBuf) -> Result<Self, String> {
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let catalog = Self {
            conn: Mutex::new(conn),
        };
        catalog.init_schema()?;
        Ok(catalog)
    }

    /// Creates tables, indexes, and FTS virtual table if they don't exist.
    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sounds (
                id TEXT PRIMARY KEY,
                event_name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                category TEXT NOT NULL,
                unit_type TEXT,
                subcategory TEXT,
                duration_ms INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                tags TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_sounds_category ON sounds(category);
            CREATE INDEX IF NOT EXISTS idx_sounds_unit_type ON sounds(unit_type);

            CREATE VIRTUAL TABLE IF NOT EXISTS sounds_fts USING fts5(
                event_name, display_name, tags,
                content='sounds',
                content_rowid='rowid'
            );

            -- Triggers to keep FTS in sync with sounds table
            CREATE TRIGGER IF NOT EXISTS sounds_ai AFTER INSERT ON sounds BEGIN
                INSERT INTO sounds_fts(rowid, event_name, display_name, tags)
                VALUES (new.rowid, new.event_name, new.display_name, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS sounds_ad AFTER DELETE ON sounds BEGIN
                INSERT INTO sounds_fts(sounds_fts, rowid, event_name, display_name, tags)
                VALUES ('delete', old.rowid, old.event_name, old.display_name, old.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS sounds_au AFTER UPDATE ON sounds BEGIN
                INSERT INTO sounds_fts(sounds_fts, rowid, event_name, display_name, tags)
                VALUES ('delete', old.rowid, old.event_name, old.display_name, old.tags);
                INSERT INTO sounds_fts(rowid, event_name, display_name, tags)
                VALUES (new.rowid, new.event_name, new.display_name, new.tags);
            END;

            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS music_tracks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                file_path TEXT NOT NULL,
                duration_secs REAL DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_music_tracks_title ON music_tracks(title);
        "#,
        )
        .map_err(|e| format!("Failed to create schema: {}", e))?;

        // Migration: Add is_favorite column if it doesn't exist
        // SQLite doesn't have ALTER TABLE ADD COLUMN IF NOT EXISTS,
        // so we check if the column exists first
        let has_favorite_column: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('sounds') WHERE name = 'is_favorite'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if !has_favorite_column {
            conn.execute_batch(
                r#"
                ALTER TABLE sounds ADD COLUMN is_favorite INTEGER DEFAULT 0 NOT NULL;
                CREATE INDEX IF NOT EXISTS idx_sounds_favorite ON sounds(is_favorite);
                "#,
            )
            .map_err(|e| format!("Failed to add is_favorite column: {}", e))?;
        }

        Ok(())
    }

    /// Runs one-time data migrations. Should be called on app startup.
    pub fn run_migrations(&self) -> Result<(), String> {
        self.migrate_remove_excluded_sounds()?;
        Ok(())
    }

    /// Migration: Remove sounds matching exclusion patterns (unreleased content).
    /// Runs once, tracked via metadata table.
    fn migrate_remove_excluded_sounds(&self) -> Result<(), String> {
        const MIGRATION_KEY: &str = "migration_removed_excluded_sounds_v1";
        const EXCLUSION_PATTERNS: &[&str] =
            &["jungle", "huns", "yuezhi", "india", "migration", "monkey"];

        // Check if migration already ran
        if self.get_metadata(MIGRATION_KEY)?.is_some() {
            return Ok(());
        }

        // Delete matching sounds and get file paths
        let file_paths = self.delete_sounds_matching_patterns(EXCLUSION_PATTERNS)?;

        // Delete files from disk
        for path in file_paths {
            let _ = std::fs::remove_file(&path);
        }

        // Mark migration as complete
        self.set_metadata(MIGRATION_KEY, "done")?;

        Ok(())
    }

    /// Gets a value from the metadata table.
    fn get_metadata(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let result = conn.query_row(
            "SELECT value FROM metadata WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get metadata: {}", e)),
        }
    }

    /// Sets a value in the metadata table.
    fn set_metadata(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| format!("Failed to set metadata: {}", e))?;
        Ok(())
    }

    /// Searches sounds using FTS5 with optional category/unit_type filters.
    /// Empty query returns all sounds (filtered by category/unit_type if provided).
    pub fn search_sounds(
        &self,
        query: &str,
        category: Option<&str>,
        unit_type: Option<&str>,
    ) -> Result<Vec<Sound>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let trimmed_query = query.trim();
        let use_fts = !trimmed_query.is_empty();

        // Build the SQL query dynamically
        let sql = if use_fts {
            let mut sql = String::from(
                "SELECT s.id, s.event_name, s.display_name, s.category,
                        s.unit_type, s.subcategory, s.duration_ms, s.file_path, s.tags, s.is_favorite
                 FROM sounds s
                 JOIN sounds_fts fts ON s.rowid = fts.rowid
                 WHERE sounds_fts MATCH ?1",
            );

            if category.is_some() {
                sql.push_str(" AND s.category = ?2");
            }
            if unit_type.is_some() {
                if category.is_some() {
                    sql.push_str(" AND s.unit_type = ?3");
                } else {
                    sql.push_str(" AND s.unit_type = ?2");
                }
            }
            sql.push_str(" ORDER BY rank LIMIT 500");
            sql
        } else {
            let mut sql = String::from(
                "SELECT s.id, s.event_name, s.display_name, s.category,
                        s.unit_type, s.subcategory, s.duration_ms, s.file_path, s.tags, s.is_favorite
                 FROM sounds s
                 WHERE 1=1",
            );

            if category.is_some() {
                sql.push_str(" AND s.category = ?1");
            }
            if unit_type.is_some() {
                if category.is_some() {
                    sql.push_str(" AND s.unit_type = ?2");
                } else {
                    sql.push_str(" AND s.unit_type = ?1");
                }
            }
            sql.push_str(" ORDER BY s.display_name ASC LIMIT 500");
            sql
        };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        // Build params based on what we have
        let rows = if use_fts {
            let fts_query = format!("{}*", trimmed_query); // Prefix search
            match (category, unit_type) {
                (Some(cat), Some(unit)) => stmt.query_map(params![fts_query, cat, unit], row_to_sound),
                (Some(cat), None) => stmt.query_map(params![fts_query, cat], row_to_sound),
                (None, Some(unit)) => stmt.query_map(params![fts_query, unit], row_to_sound),
                (None, None) => stmt.query_map(params![fts_query], row_to_sound),
            }
        } else {
            match (category, unit_type) {
                (Some(cat), Some(unit)) => stmt.query_map(params![cat, unit], row_to_sound),
                (Some(cat), None) => stmt.query_map(params![cat], row_to_sound),
                (None, Some(unit)) => stmt.query_map(params![unit], row_to_sound),
                (None, None) => stmt.query_map([], row_to_sound),
            }
        }
        .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect results: {}", e))
    }

    /// Returns all categories with their sound counts.
    pub fn get_categories(&self) -> Result<Vec<Category>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT category, COUNT(*) as count
                 FROM sounds
                 GROUP BY category
                 ORDER BY count DESC",
            )
            .map_err(|e| format!("Failed to prepare: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let count: u32 = row.get(1)?;
                Ok(Category {
                    name: format_category_name(&id),
                    id,
                    count,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect: {}", e))
    }

    /// Returns all unit types with their sound counts.
    pub fn get_unit_types(&self) -> Result<Vec<UnitType>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT unit_type, COUNT(*) as count
                 FROM sounds
                 WHERE unit_type IS NOT NULL
                 GROUP BY unit_type
                 ORDER BY unit_type ASC",
            )
            .map_err(|e| format!("Failed to prepare: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let count: u32 = row.get(1)?;
                Ok(UnitType {
                    name: id.clone(),
                    id,
                    count,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect: {}", e))
    }

    /// Inserts a sound into the catalog. FTS is updated via trigger.
    pub fn insert_sound(&self, sound: &Sound) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let tags_json = serde_json::to_string(&sound.tags)
            .map_err(|e| format!("Failed to serialize tags: {}", e))?;
        let duration_ms = (sound.duration * 1000.0) as i64;
        let is_favorite_int = if sound.is_favorite { 1 } else { 0 };

        conn.execute(
            "INSERT OR REPLACE INTO sounds
             (id, event_name, display_name, category, unit_type, subcategory,
              duration_ms, file_path, tags, is_favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                sound.id,
                sound.event_name,
                sound.display_name,
                sound.category,
                sound.unit_type,
                sound.subcategory,
                duration_ms,
                sound.file_path,
                tags_json,
                is_favorite_int,
            ],
        )
        .map_err(|e| format!("Failed to insert sound: {}", e))?;

        Ok(())
    }

    /// Toggles the favorite status of a sound. Returns the new favorite state.
    pub fn toggle_favorite(&self, sound_id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE sounds SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![sound_id],
        )
        .map_err(|e| format!("Failed to toggle favorite: {}", e))?;

        let new_state: i32 = conn
            .query_row(
                "SELECT is_favorite FROM sounds WHERE id = ?1",
                params![sound_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get new favorite state: {}", e))?;

        Ok(new_state != 0)
    }

    /// Returns count of sounds in the catalog.
    pub fn count_sounds(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let count: u64 = conn
            .query_row("SELECT COUNT(*) FROM sounds", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count: {}", e))?;
        Ok(count)
    }

    /// Returns count of favorited sounds.
    pub fn count_favorites(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let count: u64 = conn
            .query_row("SELECT COUNT(*) FROM sounds WHERE is_favorite = 1", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count favorites: {}", e))?;
        Ok(count)
    }

    /// Returns all favorited sounds.
    pub fn get_favorites(&self) -> Result<Vec<Sound>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, event_name, display_name, category, unit_type, subcategory,
                        duration_ms, file_path, tags, is_favorite
                 FROM sounds
                 WHERE is_favorite = 1
                 ORDER BY display_name ASC",
            )
            .map_err(|e| format!("Failed to prepare: {}", e))?;

        let rows = stmt
            .query_map([], row_to_sound)
            .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect: {}", e))
    }

    /// Clears all sounds from the catalog and resets migration flags.
    /// Used when rebuilding the cache.
    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // Delete all sounds (triggers will clean up FTS)
        conn.execute("DELETE FROM sounds", [])
            .map_err(|e| format!("Failed to clear sounds: {}", e))?;

        // Reset migration flags so they run again on next extraction
        conn.execute("DELETE FROM metadata", [])
            .map_err(|e| format!("Failed to clear metadata: {}", e))?;

        Ok(())
    }

    /// Deletes sounds matching any of the given patterns (case-insensitive substring match on event_name).
    /// Returns the file paths of deleted sounds so they can be removed from disk.
    pub fn delete_sounds_matching_patterns(&self, patterns: &[&str]) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // Build WHERE clause for pattern matching
        // Using LIKE with LOWER() for case-insensitive substring matching
        let conditions: Vec<String> = patterns
            .iter()
            .map(|p| format!("LOWER(event_name) LIKE '%{}%'", p.to_lowercase()))
            .collect();

        if conditions.is_empty() {
            return Ok(Vec::new());
        }

        let where_clause = conditions.join(" OR ");

        // First, get the file paths of sounds to delete
        let select_sql = format!("SELECT file_path FROM sounds WHERE {}", where_clause);
        let mut stmt = conn
            .prepare(&select_sql)
            .map_err(|e| format!("Failed to prepare select: {}", e))?;

        let file_paths: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("Query failed: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        // Delete the sounds from the database
        let delete_sql = format!("DELETE FROM sounds WHERE {}", where_clause);
        conn.execute(&delete_sql, [])
            .map_err(|e| format!("Failed to delete sounds: {}", e))?;

        Ok(file_paths)
    }

    // ========== Music Track Methods ==========

    /// Inserts a music track into the catalog.
    pub fn insert_music_track(&self, track: &MusicTrack) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO music_tracks (id, title, file_path, duration_secs)
             VALUES (?1, ?2, ?3, ?4)",
            params![track.id, track.title, track.file_path, track.duration_secs],
        )
        .map_err(|e| format!("Failed to insert music track: {}", e))?;

        Ok(())
    }

    /// Returns all music tracks, ordered by title.
    pub fn get_music_tracks(&self) -> Result<Vec<MusicTrack>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, file_path, duration_secs
                 FROM music_tracks
                 ORDER BY title ASC",
            )
            .map_err(|e| format!("Failed to prepare: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(MusicTrack {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    file_path: row.get(2)?,
                    duration_secs: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect: {}", e))
    }

    /// Searches music tracks by title.
    pub fn search_music_tracks(&self, query: &str) -> Result<Vec<MusicTrack>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let search_pattern = format!("%{}%", query.to_lowercase());

        let mut stmt = conn
            .prepare(
                "SELECT id, title, file_path, duration_secs
                 FROM music_tracks
                 WHERE LOWER(title) LIKE ?1
                 ORDER BY title ASC
                 LIMIT 100",
            )
            .map_err(|e| format!("Failed to prepare: {}", e))?;

        let rows = stmt
            .query_map(params![search_pattern], |row| {
                Ok(MusicTrack {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    file_path: row.get(2)?,
                    duration_secs: row.get(3)?,
                })
            })
            .map_err(|e| format!("Query failed: {}", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect: {}", e))
    }

    /// Returns count of music tracks.
    pub fn count_music_tracks(&self) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let count: u64 = conn
            .query_row("SELECT COUNT(*) FROM music_tracks", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count: {}", e))?;
        Ok(count)
    }

    /// Clears all music tracks from the catalog.
    pub fn clear_music_tracks(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM music_tracks", [])
            .map_err(|e| format!("Failed to clear music tracks: {}", e))?;
        Ok(())
    }
}

/// Helper function to convert a row to a Sound struct
fn row_to_sound(row: &rusqlite::Row) -> rusqlite::Result<Sound> {
    let tags_json: Option<String> = row.get(8)?;
    let tags: Vec<String> = tags_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let duration_ms: i64 = row.get(6)?;
    let is_favorite: i32 = row.get(9)?;

    Ok(Sound {
        id: row.get(0)?,
        event_name: row.get(1)?,
        display_name: row.get(2)?,
        category: row.get(3)?,
        unit_type: row.get(4)?,
        subcategory: row.get(5)?,
        duration: duration_ms as f64 / 1000.0,
        file_path: row.get(7)?,
        tags,
        is_favorite: is_favorite != 0,
    })
}

/// Converts category ID to display name (e.g., "unit_attack" -> "Unit Attacks")
fn format_category_name(category_id: &str) -> String {
    match category_id {
        "unit_attack" => "Unit Attacks".to_string(),
        "unit_death" => "Unit Deaths".to_string(),
        "unit_hit" => "Unit Hits".to_string(),
        "unit_movement" => "Unit Movement".to_string(),
        "unit_vocal" => "Unit Vocals".to_string(),
        "combat" => "Combat".to_string(),
        "ui_event" => "UI Events".to_string(),
        "story_event" => "Story Events".to_string(),
        "calamity" => "Calamities".to_string(),
        "test" => "Test Sounds".to_string(),
        other => {
            // Convert snake_case to Title Case
            other
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

/// Gets the cross-platform path for the catalog database.
pub fn get_db_path() -> Result<PathBuf, String> {
    let project_dirs = directories::ProjectDirs::from("com", "kithara", "app")
        .ok_or_else(|| "Could not determine application data directory".to_string())?;

    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    Ok(data_dir.join("catalog.db"))
}
