//! GGUF header parser — high-level `parse_header` API.

use super::{GgufError, GgufMetadata, GgufQuantization, GgufValue, Reader};

/// Parse the GGUF header from `data` and extract model metadata.
///
/// Only the first few kilobytes of the file are needed; the tensor data itself
/// is *not* read.  `data` must contain at least the full header + KV section.
pub fn parse_header(data: &[u8]) -> Result<GgufMetadata, GgufError> {
    if data.len() < 16 {
        return Err(GgufError::TooShort);
    }

    let mut reader = Reader::new(data);

    // 1. Magic
    let magic = reader.read_u32()?;
    if magic != super::GGUF_MAGIC {
        return Err(GgufError::InvalidMagic(magic));
    }

    // 2. Version
    let version = reader.read_u32()?;
    if !(2..=3).contains(&version) {
        return Err(GgufError::UnsupportedVersion(version));
    }

    // 3. Tensor count & metadata KV count
    let _tensor_count = reader.read_u64()?;
    let kv_count = reader.read_u64()?;

    // 4. Walk KV pairs and collect the ones we care about.
    let mut architecture = String::new();
    let mut context_length: u64 = 0;
    let mut embedding_length: u64 = 0;
    let mut vocab_size: u64 = 0;
    let mut quantization_raw: Option<u32> = None;

    for _ in 0..kv_count {
        let key = reader.read_string()?;
        let value_type = reader.read_u32()?;
        let value = read_value(&mut reader, value_type)?;

        // Match well-known keys.
        if key == "general.architecture" {
            if let Some(s) = value.as_str() {
                architecture = s.to_string();
            }
        } else if key.ends_with(".context_length") {
            if let Some(v) = value.as_u64() {
                context_length = v;
            }
        } else if key.ends_with(".embedding_length") {
            if let Some(v) = value.as_u64() {
                embedding_length = v;
            }
        } else if key.ends_with(".vocab_size") || key == "tokenizer.ggml.vocab_size" {
            if let Some(v) = value.as_u64() {
                vocab_size = v;
            }
        } else if key == "general.file_type" {
            if let Some(v) = value.as_u64() {
                quantization_raw = Some(v as u32);
            }
        }
    }

    let quantization = quantization_raw
        .map(GgufQuantization::from)
        .unwrap_or(GgufQuantization::Unknown(0));

    Ok(GgufMetadata {
        architecture,
        context_length,
        quantization,
        embedding_length,
        vocab_size,
        file_size: data.len() as u64,
    })
}

fn read_value(reader: &mut Reader, type_id: u32) -> Result<GgufValue, GgufError> {
    match type_id {
        super::GGUF_TYPE_UINT8 => {
            reader.read_u8()?;
            Ok(GgufValue::Other)
        }
        super::GGUF_TYPE_INT8 => {
            reader.read_u8()?;
            Ok(GgufValue::Other)
        }
        super::GGUF_TYPE_UINT16 => {
            if reader.remaining() < 2 {
                return Err(GgufError::UnexpectedEof);
            }
            reader.pos += 2;
            Ok(GgufValue::Other)
        }
        super::GGUF_TYPE_INT16 => {
            if reader.remaining() < 2 {
                return Err(GgufError::UnexpectedEof);
            }
            reader.pos += 2;
            Ok(GgufValue::Other)
        }
        super::GGUF_TYPE_UINT32 => Ok(GgufValue::U32(reader.read_u32()?)),
        super::GGUF_TYPE_INT32 => Ok(GgufValue::I32(reader.read_i32()?)),
        super::GGUF_TYPE_FLOAT32 => Ok(GgufValue::F32(reader.read_f32()?)),
        super::GGUF_TYPE_BOOL => {
            reader.read_bool()?;
            Ok(GgufValue::Bool(()))
        }
        super::GGUF_TYPE_STRING => Ok(GgufValue::Str(reader.read_string()?)),
        super::GGUF_TYPE_ARRAY => {
            let elem_type = reader.read_u32()?;
            let count = reader.read_u64()? as usize;
            for _ in 0..count {
                read_value(reader, elem_type)?;
            }
            Ok(GgufValue::Array(()))
        }
        super::GGUF_TYPE_UINT64 => Ok(GgufValue::U64(reader.read_u64()?)),
        super::GGUF_TYPE_INT64 => Ok(GgufValue::I64(reader.read_i64()?)),
        super::GGUF_TYPE_FLOAT64 => Ok(GgufValue::F64(reader.read_f64()?)),
        _ => Err(GgufError::UnexpectedEof),
    }
}
