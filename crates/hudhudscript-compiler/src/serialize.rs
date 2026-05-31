//! Bytecode serialization to `.hudc` file format (Issue #605)
//!
//! Provides binary serialization/deserialization of compiled bytecode with
//! a header containing magic bytes, version, timestamp, counts and a CRC32
//! checksum for integrity verification.

use crate::bytecode::{Instruction, Value16, BYTECODE_VERSION};
use crate::error::{compile_codes, CompileResult};
use std::path::Path;

/// Magic bytes identifying a `.hudc` file: "HUDC" in ASCII.
pub const MAGIC_BYTES: [u8; 4] = [0x48, 0x55, 0x44, 0x43];

/// Header for the `.hudc` binary format.
#[derive(Debug, Clone)]
pub struct HudcHeader {
    /// Magic bytes (must equal [`MAGIC_BYTES`]).
    pub magic: [u8; 4],
    /// Bytecode format version.
    pub version: u32,
    /// Unix timestamp (seconds since epoch) when the file was produced.
    pub timestamp: u64,
    /// Number of instructions in the payload.
    pub instruction_count: u32,
    /// Number of constants in the payload.
    pub constant_count: u32,
    /// CRC32 checksum of the serialized payload (instructions + constants).
    pub checksum: u32,
}

/// Fixed size of the header in bytes.
const HEADER_SIZE: usize = 4 + 4 + 8 + 4 + 4 + 4; // 28 bytes

impl HudcHeader {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE);
        buf.extend_from_slice(&self.magic);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.timestamp.to_le_bytes());
        buf.extend_from_slice(&self.instruction_count.to_le_bytes());
        buf.extend_from_slice(&self.constant_count.to_le_bytes());
        buf.extend_from_slice(&self.checksum.to_le_bytes());
        buf
    }

    fn from_bytes(data: &[u8]) -> CompileResult<Self> {
        if data.len() < HEADER_SIZE {
            return Err(compile_codes::invalid_bytecode(
                "File too small for .hudc header",
            ));
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&data[0..4]);
        if magic != MAGIC_BYTES {
            return Err(compile_codes::invalid_bytecode(
                "Invalid magic bytes — not a .hudc file",
            ));
        }

        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let timestamp = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let instruction_count = u32::from_le_bytes(data[16..20].try_into().unwrap());
        let constant_count = u32::from_le_bytes(data[20..24].try_into().unwrap());
        let checksum = u32::from_le_bytes(data[24..28].try_into().unwrap());

        Ok(Self {
            magic,
            version,
            timestamp,
            instruction_count,
            constant_count,
            checksum,
        })
    }
}

// ---------------------------------------------------------------------------
// CRC32 (IEEE) — minimal implementation so we don't need an extra crate
// ---------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Serialize instructions and constants into the `.hudc` binary format.
///
/// As of BYTECODE_VERSION v8 the payload encoding is postcard (varint,
/// 40-60% smaller than bincode) — see PERF-49 / Audit v3 Finding 15.1.
/// The `.hudc` header wrapper (magic + version + CRC) is unchanged.
pub fn serialize_bytecode(instructions: &[Instruction], constants: &[Value16]) -> Vec<u8> {
    let payload = postcard::to_stdvec(&(instructions, constants))
        .expect("postcard serialization should not fail for valid bytecode");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let header = HudcHeader {
        magic: MAGIC_BYTES,
        version: BYTECODE_VERSION,
        timestamp: now,
        instruction_count: instructions.len() as u32,
        constant_count: constants.len() as u32,
        checksum: crc32(&payload),
    };

    let mut out = header.to_bytes();
    out.extend_from_slice(&payload);
    out
}

/// Deserialize instructions and constants from `.hudc` binary data.
pub fn deserialize_bytecode(data: &[u8]) -> CompileResult<(Vec<Instruction>, Vec<Value16>)> {
    let header = HudcHeader::from_bytes(data)?;

    if header.version != BYTECODE_VERSION {
        return Err(compile_codes::invalid_bytecode(format!(
            "Bytecode version mismatch: expected {}, got {}",
            BYTECODE_VERSION, header.version
        )));
    }

    let payload = &data[HEADER_SIZE..];
    let actual_checksum = crc32(payload);
    if actual_checksum != header.checksum {
        return Err(compile_codes::invalid_bytecode(format!(
            "Checksum mismatch: expected {:08x}, got {:08x}",
            header.checksum, actual_checksum
        )));
    }

    let (instructions, constants): (Vec<Instruction>, Vec<Value16>) = postcard::from_bytes(payload)
        .map_err(|e| {
            compile_codes::invalid_bytecode(format!("Failed to deserialize payload: {e}"))
        })?;

    if instructions.len() != header.instruction_count as usize {
        return Err(compile_codes::invalid_bytecode(format!(
            "Instruction count mismatch: header says {}, payload has {}",
            header.instruction_count,
            instructions.len()
        )));
    }

    if constants.len() != header.constant_count as usize {
        return Err(compile_codes::invalid_bytecode(format!(
            "Constant count mismatch: header says {}, payload has {}",
            header.constant_count,
            constants.len()
        )));
    }

    Ok((instructions, constants))
}

/// Serialize and write a `.hudc` file to disk.
pub fn save_hudc(
    path: &Path,
    instructions: &[Instruction],
    constants: &[Value16],
) -> CompileResult<()> {
    let data = serialize_bytecode(instructions, constants);
    std::fs::write(path, data)
        .map_err(|e| compile_codes::runtime_error(format!("Failed to write .hudc file: {e}")))
}

/// Read and deserialize a `.hudc` file from disk.
pub fn load_hudc(path: &Path) -> CompileResult<(Vec<Instruction>, Vec<Value16>)> {
    let data = std::fs::read(path)
        .map_err(|e| compile_codes::runtime_error(format!("Failed to read .hudc file: {e}")))?;
    deserialize_bytecode(&data)
}

// Tests moved to hudhud-script-tests/tests/compiler_test_inline.rs
