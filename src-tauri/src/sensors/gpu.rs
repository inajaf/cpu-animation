//! GPU usage/temperature.
//!
//! - Windows/NVIDIA: NVML (official driver API) — utilization, VRAM, temp.
//! - macOS: the IOAccelerator registry entry publishes `PerformanceStatistics`
//!   with `Device Utilization %` and unified-memory in-use bytes; read via
//!   `ioreg -a` and plist parsing (no root, works on Apple Silicon). GPU temp
//!   has no public API on macOS (SMC keys are private) and stays `None`.
//! - AMD/Intel on Windows: no maintained public API — `Unavailable`.

use crate::state::UsageMetric;

#[cfg(windows)]
mod imp {
    use crate::state::UsageMetric;
    use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
    use nvml_wrapper::Nvml;
    use std::sync::OnceLock;

    static NVML: OnceLock<Option<Nvml>> = OnceLock::new();

    pub fn sample() -> UsageMetric {
        let nvml = NVML.get_or_init(|| Nvml::init().ok());
        let Some(nvml) = nvml.as_ref() else {
            return UsageMetric::default();
        };
        let Ok(device) = nvml.device_by_index(0) else {
            return UsageMetric::default();
        };
        let percent = device.utilization_rates().ok().map(|u| u.gpu as f32);
        let temp_celsius = device
            .temperature(TemperatureSensor::Gpu)
            .ok()
            .map(|t| t as f32);
        let (total_bytes, used_bytes) = device
            .memory_info()
            .ok()
            .map(|m| (Some(m.total), Some(m.used)))
            .unwrap_or((None, None));

        UsageMetric {
            percent,
            used_bytes,
            total_bytes,
            temp_celsius,
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use crate::state::UsageMetric;
    use std::process::Command;

    /// Spawning `ioreg` once per poll tick costs a few ms — acceptable at
    /// 1/3s cadence. A native IOKit binding would avoid the spawn but pulls
    /// in a large unsafe surface for the same numbers.
    pub fn sample() -> UsageMetric {
        let Ok(output) = Command::new("ioreg")
            .args(["-r", "-d", "1", "-c", "IOAccelerator", "-a"])
            .output()
        else {
            return UsageMetric::default();
        };
        if !output.status.success() || output.stdout.is_empty() {
            return UsageMetric::default();
        }

        let Ok(entries) = plist::from_bytes::<Vec<plist::Dictionary>>(&output.stdout) else {
            return UsageMetric::default();
        };

        // Integrated Macs expose one accelerator; take the first entry that
        // actually reports utilization.
        for entry in &entries {
            let Some(stats) = entry
                .get("PerformanceStatistics")
                .and_then(|v| v.as_dictionary())
            else {
                continue;
            };
            let Some(percent) = stats
                .get("Device Utilization %")
                .and_then(|v| v.as_signed_integer())
            else {
                continue;
            };

            let used_bytes = stats
                .get("In use system memory")
                .and_then(|v| v.as_signed_integer())
                .map(|v| v.max(0) as u64);

            return UsageMetric {
                percent: Some((percent.clamp(0, 100)) as f32),
                used_bytes,
                // Unified memory: the GPU shares system RAM, so a "total"
                // would just restate the RAM card. Leave it out.
                total_bytes: None,
                temp_celsius: None,
            };
        }

        UsageMetric::default()
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod imp {
    use crate::state::UsageMetric;

    pub fn sample() -> UsageMetric {
        UsageMetric::default()
    }
}

pub fn sample() -> UsageMetric {
    imp::sample()
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// Requires real hardware: asserts the IOAccelerator path yields a
    /// utilization figure on Apple Silicon Macs.
    #[test]
    fn macos_gpu_reports_utilization() {
        let metric = sample();
        assert!(
            metric.percent.is_some(),
            "expected Device Utilization % from IOAccelerator"
        );
        let p = metric.percent.unwrap();
        assert!((0.0..=100.0).contains(&p), "utilization out of range: {p}");
    }
}
