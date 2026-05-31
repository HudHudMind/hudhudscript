//! Shared OS info builtin — used by both VM and interpreter.
//!
//! Provides os.name(), arch(), version(), hostname(), username(),
//! homedir(), tmpdir(), cpus(), uptime(), pid().

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsMethodId {
    Name,
    Arch,
    Version,
    Hostname,
    Username,
    Homedir,
    Tmpdir,
    Cpus,
    Uptime,
    Pid,
}

impl std::str::FromStr for OsMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "arch" => Ok(Self::Arch),
            "version" => Ok(Self::Version),
            "hostname" => Ok(Self::Hostname),
            "username" => Ok(Self::Username),
            "homedir" => Ok(Self::Homedir),
            "tmpdir" => Ok(Self::Tmpdir),
            "cpus" => Ok(Self::Cpus),
            "uptime" => Ok(Self::Uptime),
            "pid" => Ok(Self::Pid),
            _ => Err(runtime_error(format!("Unknown os method: {}", s))),
        }
    }
}

impl OsMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Name => name(args),
            Self::Arch => arch(args),
            Self::Version => version(args),
            Self::Hostname => hostname(args),
            Self::Username => username(args),
            Self::Homedir => homedir(args),
            Self::Tmpdir => tmpdir(args),
            Self::Cpus => cpus(args),
            Self::Uptime => uptime(args),
            Self::Pid => pid(args),
        }
    }
}

pub fn name(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(std::env::consts::OS.to_string()))
}

pub fn arch(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(std::env::consts::ARCH.to_string()))
}

pub fn version(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(format!(
        "{} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    )))
}

pub fn hostname(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(get_hostname()))
}

pub fn username(_args: &[Value16]) -> HudHudResult<Value16> {
    match std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        Ok(user) => Ok(Value16::string(user)),
        Err(_) => Ok(Value16::string("unknown".to_string())),
    }
}

pub fn homedir(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
    ))
}

pub fn tmpdir(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(
        std::env::temp_dir().to_string_lossy().to_string(),
    ))
}

pub fn cpus(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(get_cpu_count()))
}

pub fn uptime(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(sysinfo::System::uptime() as f64))
}

pub fn pid(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::number(std::process::id() as f64))
}

#[cfg(unix)]
fn get_hostname() -> String {
    let mut buf = [0u8; 256];
    let result = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if result == 0 {
        let hostname = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
        hostname.to_string_lossy().to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(not(unix))]
fn get_hostname() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(unix)]
fn get_cpu_count() -> f64 {
    let cpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    if cpus > 0 {
        cpus as f64
    } else {
        1.0
    }
}

#[cfg(not(unix))]
fn get_cpu_count() -> f64 {
    std::env::var("NUMBER_OF_PROCESSORS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0)
}
