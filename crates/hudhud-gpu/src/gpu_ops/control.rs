//! GPU control / configuration operations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::utils::{make_obj, run_cmd, runtime_error, type_error, which_exists};

pub fn gpu_cuda_available(_args: &[Value16]) -> HudHudResult<Value16> {
    if which_exists("nvidia-smi") {
        if let Some(output) = run_cmd("nvidia-smi", &[]) {
            if output.contains("CUDA Version") {
                return Ok(Value16::bool_(true));
            }
        }
    }
    if which_exists("nvcc") {
        return Ok(Value16::bool_(true));
    }
    Ok(Value16::bool_(false))
}

pub fn gpu_rocm_available(_args: &[Value16]) -> HudHudResult<Value16> {
    if which_exists("rocm-smi") {
        return Ok(Value16::bool_(true));
    }
    if which_exists("rocminfo") {
        return Ok(Value16::bool_(true));
    }
    Ok(Value16::bool_(false))
}

pub fn gpu_set_visible(args: &[Value16]) -> HudHudResult<Value16> {
    let indices = args.first().and_then(|v| v.as_array()).ok_or_else(|| {
        args.first().map_or_else(
            || type_error("array", "nothing", "gpu.set_visible"),
            |v| type_error("array", v.type_name_str(), "gpu.set_visible"),
        )
    })?;

    let csv: Vec<String> = indices
        .iter()
        .filter_map(|v| {
            if let Some(n) = v.as_number() {
                Some((n as u32).to_string())
            } else {
                v.as_str().map(|s| s.to_string())
            }
        })
        .collect();
    let value = csv.join(",");
    std::env::set_var("CUDA_VISIBLE_DEVICES", &value);
    Ok(Value16::string(value))
}
