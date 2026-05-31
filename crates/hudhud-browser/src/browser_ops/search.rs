use super::*;

pub fn browser_search(args: &[Value16]) -> HudHudResult<Value16> {
    let query = match args.first() {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), "browser.search"))?
            .to_string(),
        None => return Err(runtime_error("browser.search requires a query argument")),
    };

    let engine = args
        .get(1)
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "google".to_string());

    let encoded_query = url_encode(&query);
    let search_url = match engine.as_str() {
        "google" => format!("https://www.google.com/search?q={}", encoded_query),
        "duckduckgo" | "ddg" => format!("https://duckduckgo.com/?q={}", encoded_query),
        "bing" => format!("https://www.bing.com/search?q={}", encoded_query),
        "yahoo" => format!("https://search.yahoo.com/search?p={}", encoded_query),
        other => {
            return Err(runtime_error(format!(
                "browser.search: unknown engine '{}'. Supported: google, duckduckgo, bing, yahoo",
                other
            )))
        }
    };

    if std::env::var("HUDHUD_NO_BROWSER").is_ok() || cfg!(test) {
        return Ok(Value16::string(search_url));
    }

    let status = Command::new("xdg-open").arg(&search_url).status();
    match status {
        Ok(s) if s.success() => Ok(Value16::string(search_url)),
        Ok(_) => Ok(Value16::string(search_url)),
        Err(e) => Err(runtime_error(format!(
            "browser.search: failed to launch xdg-open: {}",
            e
        ))),
    }
}

pub fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", byte));
            }
        }
    }
    encoded
}
