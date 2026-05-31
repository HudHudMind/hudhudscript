//! GPU query operations — list, usage, memory, driver, processes.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::utils::{extract_number, make_obj, run_cmd, runtime_error, type_error};

pub fn gpu_list(_args: &[Value16]) -> HudHudResult<Value16> {
    if let Some(output) = run_cmd(
        "nvidia-smi",
        &[
            "--query-gpu=index,name,memory.total,driver_version,pci.bus_id",
            "--format=csv,noheader,nounits",
        ],
    ) {
        let gpus: Vec<Value16> = output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(5, ',').map(|s| s.trim()).collect();
                let index = parts
                    .first()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let name = parts.get(1).unwrap_or(&"unknown").to_string();
                let mem = parts
                    .get(2)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let driver = parts.get(3).unwrap_or(&"unknown").to_string();
                let pci = parts.get(4).unwrap_or(&"unknown").to_string();
                make_obj(vec![
                    ("index", Value16::number(index)),
                    ("name", Value16::string(name)),
                    ("memory_total_mb", Value16::number(mem)),
                    ("driver_version", Value16::string(driver)),
                    ("pci_bus", Value16::string(pci)),
                ])
            })
            .collect();
        return Ok(Value16::array(gpus));
    }

    if let Some(output) = run_cmd("rocm-smi", &["--showid", "--showproductname", "--csv"]) {
        let gpus: Vec<Value16> = output
            .lines()
            .skip(1)
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, line)| {
                let parts: Vec<&str> = line.splitn(3, ',').map(|s| s.trim()).collect();
                let name = parts.get(1).unwrap_or(&"AMD GPU").to_string();
                make_obj(vec![
                    ("index", Value16::number(i as f64)),
                    ("name", Value16::string(name)),
                    ("memory_total_mb", Value16::number(0.0)),
                    ("driver_version", Value16::string("amd".to_string())),
                    (
                        "pci_bus",
                        Value16::string(parts.first().unwrap_or(&"").to_string()),
                    ),
                ])
            })
            .collect();
        if !gpus.is_empty() {
            return Ok(Value16::array(gpus));
        }
    }

    if let Some(output) = run_cmd("lspci", &[]) {
        let gpus: Vec<Value16> = output
            .lines()
            .filter(|l| {
                let lower = l.to_lowercase();
                lower.contains("vga")
                    || lower.contains("3d controller")
                    || lower.contains("display")
            })
            .enumerate()
            .map(|(i, line)| {
                let name = line.split(':').nth(2).unwrap_or(line).trim().to_string();
                let pci = line.split_whitespace().next().unwrap_or("").to_string();
                make_obj(vec![
                    ("index", Value16::number(i as f64)),
                    ("name", Value16::string(name)),
                    ("memory_total_mb", Value16::number(0.0)),
                    ("driver_version", Value16::string("unknown".to_string())),
                    ("pci_bus", Value16::string(pci)),
                ])
            })
            .collect();
        return Ok(Value16::array(gpus));
    }

    Err(runtime_error(
        "gpu.list: no GPU detection tools available (install nvidia-smi, rocm-smi, or lspci)",
    ))
}

pub fn gpu_usage(args: &[Value16]) -> HudHudResult<Value16> {
    let index = args
        .first()
        .and_then(|v| v.as_number())
        .map(|n| n as u32)
        .ok_or_else(|| {
            args.first().map_or_else(
                || type_error("number", "nothing", "gpu.usage"),
                |v| type_error("number", v.type_name_str(), "gpu.usage"),
            )
        })?;

    let id_arg = format!("--id={}", index);
    if let Some(output) = run_cmd(
        "nvidia-smi",
        &[
            &id_arg,
            "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ],
    ) {
        let line = output.lines().next().unwrap_or("");
        let parts: Vec<&str> = line.splitn(5, ',').map(|s| s.trim()).collect();
        let util = parts
            .first()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let mem_used = parts
            .get(1)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let mem_total = parts
            .get(2)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let temp = parts
            .get(3)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let power = parts
            .get(4)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        return Ok(make_obj(vec![
            ("gpu_utilization", Value16::number(util)),
            ("memory_used_mb", Value16::number(mem_used)),
            ("memory_total_mb", Value16::number(mem_total)),
            ("temperature_c", Value16::number(temp)),
            ("power_watts", Value16::number(power)),
        ]));
    }

    if let Some(output) = run_cmd(
        "rocm-smi",
        &[
            "-d",
            &index.to_string(),
            "--showuse",
            "--showtemp",
            "--showpower",
        ],
    ) {
        let mut util = 0.0;
        let mut temp = 0.0;
        let mut power = 0.0;
        for line in output.lines() {
            let lower = line.to_lowercase();
            if lower.contains("gpu use") || lower.contains("gpu utilization") {
                if let Some(val) = extract_number(line) {
                    util = val;
                }
            }
            if lower.contains("temperature") {
                if let Some(val) = extract_number(line) {
                    temp = val;
                }
            }
            if lower.contains("power") {
                if let Some(val) = extract_number(line) {
                    power = val;
                }
            }
        }
        return Ok(make_obj(vec![
            ("gpu_utilization", Value16::number(util)),
            ("memory_used_mb", Value16::number(0.0)),
            ("memory_total_mb", Value16::number(0.0)),
            ("temperature_c", Value16::number(temp)),
            ("power_watts", Value16::number(power)),
        ]));
    }

    Ok(make_obj(vec![
        ("gpu_utilization", Value16::number(0.0)),
        ("memory_used_mb", Value16::number(0.0)),
        ("memory_total_mb", Value16::number(0.0)),
        ("temperature_c", Value16::number(0.0)),
        ("power_watts", Value16::number(0.0)),
    ]))
}

pub fn gpu_memory(_args: &[Value16]) -> HudHudResult<Value16> {
    if let Some(output) = run_cmd(
        "nvidia-smi",
        &[
            "--query-gpu=memory.used,memory.total,memory.free",
            "--format=csv,noheader,nounits",
        ],
    ) {
        let mut total_used = 0.0_f64;
        let mut total_total = 0.0_f64;
        let mut total_free = 0.0_f64;
        for line in output.lines().filter(|l| !l.trim().is_empty()) {
            let parts: Vec<&str> = line.splitn(3, ',').map(|s| s.trim()).collect();
            total_used += parts
                .first()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            total_total += parts
                .get(1)
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            total_free += parts
                .get(2)
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
        }
        return Ok(make_obj(vec![
            ("used_mb", Value16::number(total_used)),
            ("total_mb", Value16::number(total_total)),
            ("free_mb", Value16::number(total_free)),
        ]));
    }
    Ok(make_obj(vec![
        ("used_mb", Value16::number(0.0)),
        ("total_mb", Value16::number(0.0)),
        ("free_mb", Value16::number(0.0)),
    ]))
}

pub fn gpu_driver(_args: &[Value16]) -> HudHudResult<Value16> {
    if let Some(output) = run_cmd(
        "nvidia-smi",
        &["--query-gpu=driver_version", "--format=csv,noheader"],
    ) {
        let version = output.lines().next().unwrap_or("").trim().to_string();
        if !version.is_empty() {
            return Ok(make_obj(vec![
                ("name", Value16::string("nvidia".to_string())),
                ("version", Value16::string(version)),
            ]));
        }
    }
    if let Some(output) = run_cmd("rocm-smi", &["--showdriverversion"]) {
        for line in output.lines() {
            let lower = line.to_lowercase();
            if lower.contains("driver version") || lower.contains("driver") {
                let version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                if !version.is_empty() {
                    return Ok(make_obj(vec![
                        ("name", Value16::string("amd".to_string())),
                        ("version", Value16::string(version)),
                    ]));
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    if let Some(output) = run_cmd("modinfo", &["i915"]) {
        let mut version = String::new();
        for line in output.lines() {
            if line.starts_with("version:") || line.starts_with("vermagic:") {
                version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                break;
            }
        }
        if !version.is_empty() {
            return Ok(make_obj(vec![
                ("name", Value16::string("intel".to_string())),
                ("version", Value16::string(version)),
            ]));
        }
    }
    Ok(make_obj(vec![
        ("name", Value16::string("unknown".to_string())),
        ("version", Value16::string("unknown".to_string())),
    ]))
}

pub fn gpu_processes(_args: &[Value16]) -> HudHudResult<Value16> {
    if let Some(output) = run_cmd(
        "nvidia-smi",
        &[
            "--query-compute-apps=pid,process_name,used_gpu_memory",
            "--format=csv,noheader,nounits",
        ],
    ) {
        let procs: Vec<Value16> = output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(3, ',').map(|s| s.trim()).collect();
                let pid = parts
                    .first()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let name = parts.get(1).unwrap_or(&"unknown").to_string();
                let mem = parts
                    .get(2)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                make_obj(vec![
                    ("pid", Value16::number(pid)),
                    ("name", Value16::string(name)),
                    ("gpu_memory_mb", Value16::number(mem)),
                ])
            })
            .collect();
        return Ok(Value16::array(procs));
    }
    Err(runtime_error(
        "gpu.processes: nvidia-smi not available — cannot list GPU processes",
    ))
}
