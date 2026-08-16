//! Shared Math builtin — used by both VM and interpreter.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

fn get_num(args: &[Value16], idx: usize, method_name: &str) -> HudHudResult<f64> {
    args.get(idx)
        .and_then(|v| v.as_number().or_else(|| v.as_int().map(|i| i as f64)))
        .ok_or_else(|| runtime_error(format!("Math.{}() expects a number argument", method_name)))
}

pub fn abs(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "abs")?.abs()))
}

pub fn sqrt(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "sqrt")?.sqrt()))
}

pub fn floor(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "floor")?.floor()))
}

pub fn ceil(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "ceil")?.ceil()))
}

pub fn round(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "round")?.round()))
}

pub fn sign(args: &[Value16]) -> HudHudResult<Value16> {
    let n = get_num(args, 0, "sign")?;
    Ok(Value16::number(if n > 0.0 {
        1.0
    } else if n < 0.0 {
        -1.0
    } else {
        0.0
    }))
}

pub fn trunc(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "trunc")?.trunc()))
}

pub fn sin(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "sin")?.sin()))
}

pub fn cos(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "cos")?.cos()))
}

pub fn tan(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "tan")?.tan()))
}

pub fn asin(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "asin")?.asin()))
}

pub fn acos(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "acos")?.acos()))
}

pub fn atan(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "atan")?.atan()))
}

pub fn log(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "log")?.ln()))
}

pub fn log10(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "log10")?.log10()))
}

pub fn log2(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "log2")?.log2()))
}

pub fn exp(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_num(args, 0, "exp")?.exp()))
}

pub fn pow(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(
        get_num(args, 0, "pow")?.powf(get_num(args, 1, "pow")?),
    ))
}

pub fn min(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(
        get_num(args, 0, "min")?.min(get_num(args, 1, "min")?),
    ))
}

pub fn max(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(
        get_num(args, 0, "max")?.max(get_num(args, 1, "max")?),
    ))
}

pub fn atan2(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(
        get_num(args, 0, "atan2")?.atan2(get_num(args, 1, "atan2")?),
    ))
}

pub fn hypot(args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(
        get_num(args, 0, "hypot")?.hypot(get_num(args, 1, "hypot")?),
    ))
}

pub fn clamp(args: &[Value16]) -> HudHudResult<Value16> {
    let val = get_num(args, 0, "clamp")?;
    let min = get_num(args, 1, "clamp")?;
    let max = get_num(args, 2, "clamp")?;
    Ok(Value16::number(val.max(min).min(max)))
}

pub fn random(_args: &[Value16]) -> HudHudResult<Value16> {
    use std::collections::hash_map::RandomState;
    use std::hash::BuildHasher;
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let hash = RandomState::new().hash_one(time);
    let random = (hash as f64) / (u64::MAX as f64);
    Ok(Value16::number(random))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathMethodId {
    Abs,
    Sqrt,
    Floor,
    Ceil,
    Round,
    Sign,
    Trunc,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Log,
    Log10,
    Log2,
    Exp,
    Pow,
    PowMod,
    Min,
    Max,
    Atan2,
    Hypot,
    Clamp,
    Random,
}

impl std::str::FromStr for MathMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "abs" => Ok(Self::Abs),
            "sqrt" => Ok(Self::Sqrt),
            "floor" => Ok(Self::Floor),
            "ceil" => Ok(Self::Ceil),
            "round" => Ok(Self::Round),
            "sign" => Ok(Self::Sign),
            "trunc" => Ok(Self::Trunc),
            "sin" => Ok(Self::Sin),
            "cos" => Ok(Self::Cos),
            "tan" => Ok(Self::Tan),
            "asin" => Ok(Self::Asin),
            "acos" => Ok(Self::Acos),
            "atan" => Ok(Self::Atan),
            "log" => Ok(Self::Log),
            "log10" => Ok(Self::Log10),
            "log2" => Ok(Self::Log2),
            "exp" => Ok(Self::Exp),
            "pow" => Ok(Self::Pow),
            "powmod" => Ok(Self::PowMod),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "atan2" => Ok(Self::Atan2),
            "hypot" => Ok(Self::Hypot),
            "clamp" => Ok(Self::Clamp),
            "random" => Ok(Self::Random),
            _ => Err(runtime_error(format!("Unknown Math method: {}", s))),
        }
    }
}

impl MathMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Abs => abs(args),
            Self::Sqrt => sqrt(args),
            Self::Floor => floor(args),
            Self::Ceil => ceil(args),
            Self::Round => round(args),
            Self::Sign => sign(args),
            Self::Trunc => trunc(args),
            Self::Sin => sin(args),
            Self::Cos => cos(args),
            Self::Tan => tan(args),
            Self::Asin => asin(args),
            Self::Acos => acos(args),
            Self::Atan => atan(args),
            Self::Log => log(args),
            Self::Log10 => log10(args),
            Self::Log2 => log2(args),
            Self::Exp => exp(args),
            Self::Pow => pow(args),
            Self::PowMod => powmod(args),
            Self::Min => min(args),
            Self::Max => max(args),
            Self::Atan2 => atan2(args),
            Self::Hypot => hypot(args),
            Self::Clamp => clamp(args),
            Self::Random => random(args),
        }
    }
}

pub fn powmod(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() != 3 {
        return Err(runtime_error(
            "Math.powmod(base, exp, mod) requires exactly 3 arguments".to_string(),
        ));
    }
    let base = args[0]
        .as_int()
        .ok_or_else(|| runtime_error("Math.powmod: base must be an integer".to_string()))?;
    let exp = args[1]
        .as_int()
        .ok_or_else(|| runtime_error("Math.powmod: exp must be an integer".to_string()))?;
    let modulus = args[2]
        .as_int()
        .ok_or_else(|| runtime_error("Math.powmod: mod must be an integer".to_string()))?;
    if modulus <= 0 {
        return Err(runtime_error(
            "Math.powmod: mod must be positive".to_string(),
        ));
    }
    let result = modular_pow(base, exp, modulus);
    Ok(Value16::int(result))
}

fn modular_pow(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    let mut result = 1i64;
    base = base.wrapping_rem(modulus);
    while exp > 0 {
        if exp % 2 == 1 {
            result = result.wrapping_mul(base).wrapping_rem(modulus);
        }
        base = base.wrapping_mul(base).wrapping_rem(modulus);
        exp /= 2;
    }
    result
}
