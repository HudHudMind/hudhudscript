//! Printf-style format engine — single source (Kural 7).
//! Supports %s, %d, %f, %%.

/// Argument types for sprintf.
#[derive(Debug, Clone)]
pub enum FmtArg {
    Str(String),
    Int(i64),
    Float(f64),
}

/// Format a string with printf-style placeholders.
/// %s = string, %d = integer, %f = float (6 decimal places), %% = literal %.
pub fn sprintf(fmt: &str, args: &[FmtArg]) -> Result<String, String> {
    let mut result = String::new();
    let mut arg_idx = 0;
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            i += 1;
            match chars[i] {
                '%' => result.push('%'),
                's' => {
                    if arg_idx >= args.len() {
                        return Err(format!(
                            "printf: not enough arguments (need arg {})",
                            arg_idx + 1
                        ));
                    }
                    match &args[arg_idx] {
                        FmtArg::Str(s) => result.push_str(s),
                        other => {
                            return Err(format!(
                                "printf: arg {} expected string, got {:?}",
                                arg_idx + 1,
                                other
                            ))
                        }
                    }
                    arg_idx += 1;
                }
                'd' => {
                    if arg_idx >= args.len() {
                        return Err(format!(
                            "printf: not enough arguments (need arg {})",
                            arg_idx + 1
                        ));
                    }
                    match &args[arg_idx] {
                        FmtArg::Int(n) => result.push_str(&n.to_string()),
                        other => {
                            return Err(format!(
                                "printf: arg {} expected int, got {:?}",
                                arg_idx + 1,
                                other
                            ))
                        }
                    }
                    arg_idx += 1;
                }
                'f' => {
                    if arg_idx >= args.len() {
                        return Err(format!(
                            "printf: not enough arguments (need arg {})",
                            arg_idx + 1
                        ));
                    }
                    match &args[arg_idx] {
                        FmtArg::Float(n) => result.push_str(&format!("{:.6}", n)),
                        other => {
                            return Err(format!(
                                "printf: arg {} expected float, got {:?}",
                                arg_idx + 1,
                                other
                            ))
                        }
                    }
                    arg_idx += 1;
                }
                _ => return Err(format!("printf: unknown format specifier '%{}'", chars[i])),
            }
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }
    Ok(result)
}
