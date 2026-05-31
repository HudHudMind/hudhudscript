//! Shared Statistics builtin — used by both VM and interpreter.

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

/// Extract a Vec<f64> from a SharedValue array.
fn extract_numbers(v: &Value16) -> HudHudResult<Vec<f64>> {
    v.as_array()
        .ok_or_else(|| runtime_error("stats: expected array"))?
        .iter()
        .map(|x| {
            x.as_number()
                .ok_or_else(|| runtime_error("stats: expected number in array"))
        })
        .collect()
}

/// Enum identifying each Stats operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatsMethodId {
    Mean,
    Median,
    Variance,
    StdDev,
    Min,
    Max,
    Quantile,
    NormalPdf,
    NormalCdf,
    UniformPdf,
    UniformCdf,
}

impl std::str::FromStr for StatsMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mean" => Ok(Self::Mean),
            "median" => Ok(Self::Median),
            "variance" => Ok(Self::Variance),
            "std_dev" => Ok(Self::StdDev),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "quantile" => Ok(Self::Quantile),
            "normal_pdf" => Ok(Self::NormalPdf),
            "normal_cdf" => Ok(Self::NormalCdf),
            "uniform_pdf" => Ok(Self::UniformPdf),
            "uniform_cdf" => Ok(Self::UniformCdf),
            _ => Err(runtime_error(format!("Unknown stats method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for Stats operations.
pub fn dispatch(method: StatsMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        StatsMethodId::Mean => {
            if args.is_empty() {
                return Err(runtime_error("stats.mean() expects 1 argument"));
            }
            let nums = extract_numbers(&args[0])?;
            if nums.is_empty() {
                return Ok(Value16::number(0.0));
            }
            Ok(Value16::number(
                nums.iter().sum::<f64>() / nums.len() as f64,
            ))
        }
        StatsMethodId::Median => {
            if args.is_empty() {
                return Err(runtime_error("stats.median() expects 1 argument"));
            }
            let mut nums = extract_numbers(&args[0])?;
            if nums.is_empty() {
                return Ok(Value16::number(0.0));
            }
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = nums.len() / 2;
            if nums.len() % 2 == 0 {
                Ok(Value16::number((nums[mid - 1] + nums[mid]) / 2.0))
            } else {
                Ok(Value16::number(nums[mid]))
            }
        }
        StatsMethodId::Variance => {
            if args.is_empty() {
                return Err(runtime_error("stats.variance() expects 1 argument"));
            }
            let nums = extract_numbers(&args[0])?;
            if nums.is_empty() {
                return Ok(Value16::number(0.0));
            }
            let mean = nums.iter().sum::<f64>() / nums.len() as f64;
            let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
            Ok(Value16::number(var))
        }
        StatsMethodId::StdDev => {
            if args.is_empty() {
                return Err(runtime_error("stats.std_dev() expects 1 argument"));
            }
            let nums = extract_numbers(&args[0])?;
            if nums.is_empty() {
                return Ok(Value16::number(0.0));
            }
            let mean = nums.iter().sum::<f64>() / nums.len() as f64;
            let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
            Ok(Value16::number(var.sqrt()))
        }
        StatsMethodId::Min => {
            if args.is_empty() {
                return Err(runtime_error("stats.min() expects 1 argument"));
            }
            let nums = extract_numbers(&args[0])?;
            Ok(Value16::number(
                nums.iter().cloned().fold(f64::INFINITY, f64::min),
            ))
        }
        StatsMethodId::Max => {
            if args.is_empty() {
                return Err(runtime_error("stats.max() expects 1 argument"));
            }
            let nums = extract_numbers(&args[0])?;
            Ok(Value16::number(
                nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            ))
        }
        StatsMethodId::Quantile => {
            if args.len() < 2 {
                return Err(runtime_error("stats.quantile() expects 2 arguments"));
            }
            let mut nums = extract_numbers(&args[0])?;
            let q = args[1]
                .as_number()
                .ok_or_else(|| runtime_error("stats.quantile: q must be a number"))?;
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let idx = (q * (nums.len() - 1) as f64).round() as usize;
            Ok(Value16::number(nums[idx.min(nums.len() - 1)]))
        }
        StatsMethodId::NormalPdf => {
            if args.len() < 3 {
                return Err(runtime_error("stats.normal_pdf() expects 3 arguments"));
            }
            let x = args[0]
                .as_number()
                .ok_or_else(|| runtime_error("stats.normal_pdf: x must be a number"))?;
            let mean = args[1]
                .as_number()
                .ok_or_else(|| runtime_error("stats.normal_pdf: mean must be a number"))?;
            let std_dev = args[2]
                .as_number()
                .ok_or_else(|| runtime_error("stats.normal_pdf: std_dev must be a number"))?;
            if std_dev <= 0.0 {
                return Err(runtime_error(
                    "stats.normal_pdf: standard deviation must be positive",
                ));
            }
            let z = (x - mean) / std_dev;
            let pdf = (-0.5 * z * z).exp() / (std_dev * (2.0 * std::f64::consts::PI).sqrt());
            Ok(Value16::number(pdf))
        }
        StatsMethodId::NormalCdf => {
            if args.len() < 3 {
                return Err(runtime_error("stats.normal_cdf() expects 3 arguments"));
            }
            let x = args[0]
                .as_number()
                .ok_or_else(|| runtime_error("stats.normal_cdf: x must be a number"))?;
            let mean = args[1]
                .as_number()
                .ok_or_else(|| runtime_error("stats.normal_cdf: mean must be a number"))?;
            let std_dev = args[2]
                .as_number()
                .ok_or_else(|| runtime_error("stats.normal_cdf: std_dev must be a number"))?;
            if std_dev <= 0.0 {
                return Err(runtime_error(
                    "stats.normal_cdf: standard deviation must be positive",
                ));
            }
            let z = (x - mean) / (std_dev * std::f64::consts::SQRT_2);
            Ok(Value16::number(0.5 * (1.0 + erf(z))))
        }
        StatsMethodId::UniformPdf => {
            if args.len() < 3 {
                return Err(runtime_error("stats.uniform_pdf() expects 3 arguments"));
            }
            let x = args[0]
                .as_number()
                .ok_or_else(|| runtime_error("stats.uniform_pdf: x must be a number"))?;
            let min = args[1]
                .as_number()
                .ok_or_else(|| runtime_error("stats.uniform_pdf: min must be a number"))?;
            let max = args[2]
                .as_number()
                .ok_or_else(|| runtime_error("stats.uniform_pdf: max must be a number"))?;
            if min >= max {
                return Err(runtime_error(
                    "stats.uniform_pdf: uniform distribution requires min < max",
                ));
            }
            let pdf = if (min..=max).contains(&x) {
                1.0 / (max - min)
            } else {
                0.0
            };
            Ok(Value16::number(pdf))
        }
        StatsMethodId::UniformCdf => {
            if args.len() < 3 {
                return Err(runtime_error("stats.uniform_cdf() expects 3 arguments"));
            }
            let x = args[0]
                .as_number()
                .ok_or_else(|| runtime_error("stats.uniform_cdf: x must be a number"))?;
            let min = args[1]
                .as_number()
                .ok_or_else(|| runtime_error("stats.uniform_cdf: min must be a number"))?;
            let max = args[2]
                .as_number()
                .ok_or_else(|| runtime_error("stats.uniform_cdf: max must be a number"))?;
            if min >= max {
                return Err(runtime_error(
                    "stats.uniform_cdf: uniform distribution requires min < max",
                ));
            }
            let cdf = if x <= min {
                0.0
            } else if x >= max {
                1.0
            } else {
                (x - min) / (max - min)
            };
            Ok(Value16::number(cdf))
        }
    }
}

/// Execute a Stats method (kept for backward compat).

/// Abramowitz & Stegun 7.1.26 approximation to the error function.
/// Max absolute error ≈ 1.5e-7, good enough for the CDF of a normal
/// distribution where callers already tolerate statrs's internal
/// precision.  Kept private because it's purely a helper for
/// `normal_cdf`.
fn erf(x: f64) -> f64 {
    // Constants from A&S 7.1.26
    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;
    const P: f64 = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let ax = x.abs();
    let t = 1.0 / (1.0 + P * ax);
    let y = 1.0 - (((((A5 * t + A4) * t) + A3) * t + A2) * t + A1) * t * (-ax * ax).exp();
    sign * y
}
