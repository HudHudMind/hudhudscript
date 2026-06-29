use super::*;

pub fn file_type(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "media.file_type")?;

    let mut f = std::fs::File::open(&path)
        .map_err(|e| runtime_error(format!("media.file_type: {}", e)))?;
    let mut buf = [0u8; 16];
    use std::io::Read;
    let n = f
        .read(&mut buf)
        .map_err(|e| runtime_error(format!("media.file_type: {}", e)))?;
    let buf = &buf[..n];

    let detected = detect_magic(buf);

    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("type".to_string(), Value16::string(detected.0.to_string()));
    m.insert("mime".to_string(), Value16::string(detected.1.to_string()));
    Ok(Value16::object(m))
}

pub fn detect_magic(buf: &[u8]) -> (&'static str, &'static str) {
    if buf.len() < 4 {
        return ("unknown", "application/octet-stream");
    }
    if buf.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return ("PNG", "image/png");
    }
    if buf.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return ("JPEG", "image/jpeg");
    }
    if buf.starts_with(b"GIF8") {
        return ("GIF", "image/gif");
    }
    if buf.starts_with(b"BM") {
        return ("BMP", "image/bmp");
    }
    if buf.len() >= 12 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        return ("WEBP", "image/webp");
    }
    if buf.starts_with(b"%PDF") {
        return ("PDF", "application/pdf");
    }
    if buf.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        return ("ZIP", "application/zip");
    }
    if buf.starts_with(&[0x1F, 0x8B]) {
        return ("GZIP", "application/gzip");
    }
    if buf.starts_with(&[0x49, 0x49, 0x2A, 0x00]) {
        return ("TIFF", "image/tiff");
    }
    if buf.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return ("TIFF", "image/tiff");
    }
    if buf.len() >= 12 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WAVE" {
        return ("WAV", "audio/wav");
    }
    if buf.starts_with(b"ID3") || buf.starts_with(&[0xFF, 0xFB]) || buf.starts_with(&[0xFF, 0xF3]) {
        return ("MP3", "audio/mpeg");
    }
    if buf.starts_with(b"OggS") {
        return ("OGG", "audio/ogg");
    }
    if buf.starts_with(b"fLaC") {
        return ("FLAC", "audio/flac");
    }
    if buf.len() >= 8 && &buf[4..8] == b"ftyp" {
        return ("MP4", "video/mp4");
    }
    if buf.len() >= 12 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"AVI " {
        return ("AVI", "video/x-msvideo");
    }
    if buf.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        return ("MKV", "video/x-matroska");
    }
    if buf.starts_with(&[0x7F, b'E', b'L', b'F']) {
        return ("ELF", "application/x-elf");
    }
    if buf.starts_with(&[0x37, 0x7A, 0xBC, 0xAF]) {
        return ("7Z", "application/x-7z-compressed");
    }
    if buf.starts_with(b"Rar!") {
        return ("RAR", "application/x-rar-compressed");
    }
    if buf.starts_with(&[0x00, 0x61, 0x73, 0x6D]) {
        return ("WASM", "application/wasm");
    }
    ("unknown", "application/octet-stream")
}
