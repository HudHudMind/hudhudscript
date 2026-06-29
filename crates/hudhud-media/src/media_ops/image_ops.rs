use super::*;

pub fn image_info(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "media.image_info")?;

    if let Some(info) = parse_image_header(&path) {
        return Ok(info);
    }

    let output = run_cmd(
        Command::new("identify")
            .arg("-format")
            .arg("%w %h %m %b")
            .arg(&path),
        "media.image_info",
    )?;
    let text = String::from_utf8_lossy(&output);
    let parts: Vec<&str> = text.trim().splitn(4, ' ').collect();
    if parts.len() < 4 {
        return Err(runtime_error(
            "media.image_info: unexpected identify output",
        ));
    }
    let width: f64 = parts[0].parse().unwrap_or(0.0);
    let height: f64 = parts[1].parse().unwrap_or(0.0);
    let format = parts[2].to_string();
    let size_bytes = file_size(&path);

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

    let size_bytes = file_size(path);
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
    let mut marker = [0u8; 2];
    f.read_exact(&mut marker).ok()?;
    loop {
        let mut buf2 = [0u8; 2];
        f.read_exact(&mut buf2).ok()?;
        if buf2[0] != 0xFF {
            return None;
        }
        let m = buf2[1];
        if (0xC0..=0xCF).contains(&m) && m != 0xC4 && m != 0xC8 && m != 0xCC {
            let mut seg = [0u8; 7];
            f.read_exact(&mut seg).ok()?;
            let h = u16::from_be_bytes([seg[3], seg[4]]) as u32;
            let w = u16::from_be_bytes([seg[5], seg[6]]) as u32;
            let size_bytes = file_size(path);
            let mut obj = hudhudscript_bytecode::ObjMap::default();
            obj.insert("width".to_string(), Value16::number(w as f64));
            obj.insert("height".to_string(), Value16::number(h as f64));
            obj.insert("format".to_string(), Value16::string("JPEG".to_string()));
            obj.insert("size_bytes".to_string(), Value16::number(size_bytes as f64));
            return Some(Value16::object(obj));
        }
        let mut len_buf = [0u8; 2];
        f.read_exact(&mut len_buf).ok()?;
        let len = u16::from_be_bytes(len_buf) as i64;
        f.seek(SeekFrom::Current(len - 2)).ok()?;
    }
}

fn parse_webp_dimensions(path: &str) -> Option<Value16> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 30];
    let n = f.read(&mut buf).ok()?;
    if n < 30 {
        return None;
    }
    let (w, h) = if &buf[12..16] == b"VP8 " {
        let w = u16::from_le_bytes([buf[26], buf[27]]) & 0x3FFF;
        let h = u16::from_le_bytes([buf[28], buf[29]]) & 0x3FFF;
        (w as u32, h as u32)
    } else if &buf[12..16] == b"VP8L" {
        let b0 = buf[21] as u32;
        let b1 = buf[22] as u32;
        let b2 = buf[23] as u32;
        let b3 = buf[24] as u32;
        let bits = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        let w = (bits & 0x3FFF) + 1;
        let h = ((bits >> 14) & 0x3FFF) + 1;
        (w, h)
    } else {
        return None;
    };
    let size_bytes = file_size(path);
    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("width".to_string(), Value16::number(w as f64));
    m.insert("height".to_string(), Value16::number(h as f64));
    m.insert("format".to_string(), Value16::string("WEBP".to_string()));
    m.insert("size_bytes".to_string(), Value16::number(size_bytes as f64));
    Some(Value16::object(m))
}

pub fn image_resize(args: &[Value16]) -> HudHudResult<Value16> {
    let input = require_str(args, 0, "media.image_resize")?;
    let output = require_str(args, 1, "media.image_resize")?;
    let width = require_num(args, 2, "media.image_resize")? as u32;
    let height = require_num(args, 3, "media.image_resize")? as u32;

    run_cmd(
        Command::new("convert")
            .arg(&input)
            .arg("-resize")
            .arg(format!("{}x{}!", width, height))
            .arg(&output),
        "media.image_resize",
    )?;
    Ok(Value16::string(output))
}

pub fn image_convert(args: &[Value16]) -> HudHudResult<Value16> {
    let input = require_str(args, 0, "media.image_convert")?;
    let output = require_str(args, 1, "media.image_convert")?;

    run_cmd(
        Command::new("convert").arg(&input).arg(&output),
        "media.image_convert",
    )?;
    Ok(Value16::string(output))
}
