use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;

use super::util;

pub fn image_info(args: &[Value16]) -> HudHudResult<Value16> {
    let path = util::require_str(args, 0, "media.image_info")?;

    if let Some(info) = parse_image_header(&path) {
        return Ok(info);
    }

    let output = util::run_cmd(
        std::process::Command::new("identify")
            .arg("-format")
            .arg("%w %h %m %b")
            .arg(&path),
        "media.image_info",
    )?;
    let text = String::from_utf8_lossy(&output);
    let parts: Vec<&str> = text.trim().splitn(4, ' ').collect();
    if parts.len() < 4 {
        return Err(util::runtime_error(
            "media.image_info: unexpected identify output",
        ));
    }
    let width: f64 = parts[0].parse().unwrap_or(0.0);
    let height: f64 = parts[1].parse().unwrap_or(0.0);
    let format = parts[2].to_string();
    let size_bytes = util::file_size(&path);

    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("width".to_string(), Value16::number(width));
    m.insert("height".to_string(), Value16::number(height));
    m.insert("format".to_string(), Value16::string(format));
    m.insert("size_bytes".to_string(), Value16::number(size_bytes as f64));
    Ok(Value16::object(m))
}

fn parse_image_header(path: &str) -> Option<Value16> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 32];
    f.read_exact(&mut buf).ok()?;

    let (width, height, format) = if buf.starts_with(&[0x89, b'P', b'N', b'G']) {
        let w = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
        let h = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);
        (w, h, "PNG")
    } else if buf.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return parse_jpeg_dimensions(path);
    } else if buf.starts_with(b"GIF8") {
        let w = u16::from_le_bytes([buf[6], buf[7]]) as u32;
        let h = u16::from_le_bytes([buf[8], buf[9]]) as u32;
        (w, h, "GIF")
    } else if buf.starts_with(b"BM") {
        let w = u32::from_le_bytes([buf[18], buf[19], buf[20], buf[21]]);
        let h = u32::from_le_bytes([buf[22], buf[23], buf[24], buf[25]]);
        (w, h, "BMP")
    } else if buf.len() >= 16 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        return parse_webp_dimensions(path);
    } else {
        return None;
    };

    let size_bytes = util::file_size(path);
    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("width".to_string(), Value16::number(width as f64));
    m.insert("height".to_string(), Value16::number(height as f64));
    m.insert("format".to_string(), Value16::string(format.to_string()));
    m.insert("size_bytes".to_string(), Value16::number(size_bytes as f64));
    Some(Value16::object(m))
}

fn parse_jpeg_dimensions(path: &str) -> Option<Value16> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 2];
    f.read_exact(&mut buf).ok()?;
    if &buf != &[0xFF, 0xD8] {
        return None;
    }

    let mut w = 0u32;
    let mut h = 0u32;
    loop {
        let mut marker = [0u8; 2];
        if f.read_exact(&mut marker).is_err() {
            break;
        }
        if marker[0] != 0xFF {
            continue;
        }
        let mut len_bytes = [0u8; 2];
        if f.read_exact(&mut len_bytes).is_err() {
            break;
        }
        let len = u16::from_be_bytes(len_bytes) as u64;
        if marker[1] >= 0xC0
            && marker[1] <= 0xCF
            && marker[1] != 0xC4
            && marker[1] != 0xC8
            && marker[1] != 0xCC
        {
            let mut dim = [0u8; 4];
            if f.seek(SeekFrom::Current(1)).is_err() {
                break;
            }
            if f.read_exact(&mut dim).is_err() {
                break;
            }
            h = u16::from_be_bytes([dim[0], dim[1]]) as u32;
            w = u16::from_be_bytes([dim[2], dim[3]]) as u32;
            break;
        } else {
            if f.seek(SeekFrom::Current(len as i64 - 2)).is_err() {
                break;
            }
        }
    }
    if w == 0 || h == 0 {
        return None;
    }
    let size_bytes = util::file_size(path);
    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("width".to_string(), Value16::number(w as f64));
    m.insert("height".to_string(), Value16::number(h as f64));
    m.insert("format".to_string(), Value16::string("JPEG".to_string()));
    m.insert("size_bytes".to_string(), Value16::number(size_bytes as f64));
    Some(Value16::object(m))
}

fn parse_webp_dimensions(path: &str) -> Option<Value16> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 30];
    if f.read_exact(&mut buf).is_err() {
        return None;
    }
    let chunk_type = &buf[12..16];
    let (w, h) = if chunk_type == b"VP8 " {
        let bits = u32::from_le_bytes([buf[26], buf[27], buf[28], buf[29]]);
        let w = bits & 0x3FFF;
        let h = (bits >> 16) & 0x3FFF;
        (w, h)
    } else if chunk_type == b"VP8L" {
        let bits = u32::from_le_bytes([buf[21], buf[22], buf[23], buf[24]]);
        let w = (bits & 0x3FFF) + 1;
        let h = ((bits >> 14) & 0x3FFF) + 1;
        (w, h)
    } else {
        return None;
    };
    let size_bytes = util::file_size(path);
    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("width".to_string(), Value16::number(w as f64));
    m.insert("height".to_string(), Value16::number(h as f64));
    m.insert("format".to_string(), Value16::string("WEBP".to_string()));
    m.insert("size_bytes".to_string(), Value16::number(size_bytes as f64));
    Some(Value16::object(m))
}

pub fn image_resize(args: &[Value16]) -> HudHudResult<Value16> {
    let path = util::require_str(args, 0, "media.image_resize")?;
    let width = util::require_num(args, 1, "media.image_resize")? as u32;
    let height = util::require_num(args, 2, "media.image_resize")? as u32;
    let output_path = util::require_str(args, 3, "media.image_resize")?;

    let output = util::run_cmd(
        std::process::Command::new("convert")
            .arg(&path)
            .arg("-resize")
            .arg(format!("{}x{}", width, height))
            .arg(&output_path),
        "media.image_resize",
    )?;
    let _ = output;
    Ok(Value16::string(output_path))
}

pub fn image_convert(args: &[Value16]) -> HudHudResult<Value16> {
    let path = util::require_str(args, 0, "media.image_convert")?;
    let format = util::require_str(args, 1, "media.image_convert")?;
    let output_path = util::require_str(args, 2, "media.image_convert")?;

    let output = util::run_cmd(
        std::process::Command::new("convert")
            .arg(&path)
            .arg(format!("{}:{}", format, output_path)),
        "media.image_convert",
    )?;
    let _ = output;
    Ok(Value16::string(output_path))
}
