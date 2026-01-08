//! Wwise BNK soundbank parser.
//! Parses BKHD, DIDX, and DATA sections to extract embedded WEM audio.

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Represents an embedded WEM file within a BNK
#[derive(Debug, Clone)]
pub struct WemEntry {
    pub file_id: u32,
    pub offset: u32,      // Offset within DATA section
    pub size: u32,        // Size of WEM data
    pub bnk_path: PathBuf, // Source BNK file
    pub data_offset: u64,  // Absolute offset of DATA section in BNK
}

/// Chunk header in BNK file (4-byte magic + 4-byte size)
struct ChunkHeader {
    magic: [u8; 4],
    size: u32,
}

/// Parse a BNK file and return all embedded WEM entries
pub fn parse_bnk(bnk_path: &Path) -> Result<Vec<WemEntry>, String> {
    let file = File::open(bnk_path)
        .map_err(|e| format!("Failed to open BNK {}: {}", bnk_path.display(), e))?;
    let mut reader = BufReader::new(file);

    let mut entries = Vec::new();
    let mut didx_entries: Vec<(u32, u32, u32)> = Vec::new(); // (id, offset, size)
    let mut data_section_offset: u64 = 0;

    // Parse chunks until EOF
    loop {
        let chunk = match read_chunk_header(&mut reader) {
            Ok(c) => c,
            Err(_) => break, // EOF or read error - done parsing
        };

        let chunk_start = reader.stream_position().unwrap_or(0);
        let magic_str = std::str::from_utf8(&chunk.magic).unwrap_or("????");

        match magic_str {
            "BKHD" => {
                // Bank header - skip (contains version, bank ID, etc.)
            }
            "DIDX" => {
                // Data index - array of {file_id: u32, offset: u32, size: u32}
                let entry_count = chunk.size / 12;
                for _ in 0..entry_count {
                    let file_id = reader
                        .read_u32::<LittleEndian>()
                        .map_err(|e| format!("Failed to read DIDX file_id: {}", e))?;
                    let offset = reader
                        .read_u32::<LittleEndian>()
                        .map_err(|e| format!("Failed to read DIDX offset: {}", e))?;
                    let size = reader
                        .read_u32::<LittleEndian>()
                        .map_err(|e| format!("Failed to read DIDX size: {}", e))?;
                    didx_entries.push((file_id, offset, size));
                }
            }
            "DATA" => {
                // Store the absolute offset of the DATA section content
                data_section_offset = reader.stream_position().unwrap_or(0);
            }
            _ => {
                // Skip unknown chunks (HIRC, STID, ENVS, etc.)
            }
        }

        // Seek to next chunk
        if let Err(e) = reader.seek(SeekFrom::Start(chunk_start + chunk.size as u64)) {
            // If seek fails, we've probably hit EOF
            if e.kind() != std::io::ErrorKind::UnexpectedEof {
                return Err(format!("Failed to seek to next chunk: {}", e));
            }
            break;
        }
    }

    // Verify we found both DIDX and DATA sections
    if data_section_offset == 0 && !didx_entries.is_empty() {
        return Err("Found DIDX but no DATA section in BNK".to_string());
    }

    // Build WemEntry list
    for (file_id, offset, size) in didx_entries {
        entries.push(WemEntry {
            file_id,
            offset,
            size,
            bnk_path: bnk_path.to_path_buf(),
            data_offset: data_section_offset,
        });
    }

    Ok(entries)
}

/// Read chunk header (4-byte magic + 4-byte little-endian size)
fn read_chunk_header(reader: &mut BufReader<File>) -> Result<ChunkHeader, String> {
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .map_err(|e| format!("Failed to read chunk magic: {}", e))?;
    let size = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| format!("Failed to read chunk size: {}", e))?;
    Ok(ChunkHeader { magic, size })
}

/// Extract WEM bytes from BNK to a file
pub fn extract_wem_bytes(entry: &WemEntry, output_path: &Path) -> Result<(), String> {
    let file = File::open(&entry.bnk_path)
        .map_err(|e| format!("Failed to open BNK {}: {}", entry.bnk_path.display(), e))?;
    let mut reader = BufReader::new(file);

    // Calculate absolute position of the WEM data
    let absolute_offset = entry.data_offset + entry.offset as u64;

    // Seek to the WEM data
    reader
        .seek(SeekFrom::Start(absolute_offset))
        .map_err(|e| format!("Failed to seek to WEM data at offset {}: {}", absolute_offset, e))?;

    // Read WEM bytes
    let mut buffer = vec![0u8; entry.size as usize];
    reader
        .read_exact(&mut buffer)
        .map_err(|e| format!("Failed to read {} bytes of WEM data: {}", entry.size, e))?;

    // Write to output file
    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file {}: {}", output_path.display(), e))?;
    output
        .write_all(&buffer)
        .map_err(|e| format!("Failed to write WEM data: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_header_parsing() {
        // Test parsing of chunk header bytes
        let header_bytes = b"BKHD\x10\x00\x00\x00";

        // Manually parse without file I/O
        let magic: [u8; 4] = header_bytes[0..4].try_into().unwrap();
        let size = u32::from_le_bytes(header_bytes[4..8].try_into().unwrap());

        assert_eq!(&magic, b"BKHD");
        assert_eq!(size, 16);
    }
}
