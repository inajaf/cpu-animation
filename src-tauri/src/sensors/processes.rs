use crate::state::ProcessEntry;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

const TOP_N: usize = 3;

/// Top processes by CPU and by memory. Like the CPU sensor, per-process CPU
/// is a diff against the previous refresh, so `sys` must persist across
/// calls. The refresh kind is trimmed to cpu+memory to keep the per-tick
/// cost down — a full process refresh is the most expensive sample we take.
pub fn sample(sys: &mut System) -> (Vec<ProcessEntry>, Vec<ProcessEntry>) {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );

    let mut entries: Vec<ProcessEntry> = sys
        .processes()
        .values()
        .filter_map(|p| {
            let name = p.name().to_string_lossy();
            if name.is_empty() {
                return None;
            }
            Some(ProcessEntry {
                name: name.into_owned(),
                cpu_percent: p.cpu_usage(),
                mem_bytes: p.memory(),
            })
        })
        .collect();

    entries.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
    let top_cpu = entries.iter().take(TOP_N).cloned().collect();

    entries.sort_by(|a, b| b.mem_bytes.cmp(&a.mem_bytes));
    let top_mem = entries.into_iter().take(TOP_N).collect();

    (top_cpu, top_mem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_top_processes() {
        let mut sys = System::new_all();
        // First refresh baselines per-process CPU; the second yields deltas.
        sample(&mut sys);
        std::thread::sleep(std::time::Duration::from_millis(300));
        let (top_cpu, top_mem) = sample(&mut sys);

        assert_eq!(top_cpu.len(), TOP_N);
        assert_eq!(top_mem.len(), TOP_N);
        assert!(top_mem[0].mem_bytes > 0, "top memory process reports bytes");
        assert!(
            top_cpu.windows(2).all(|w| w[0].cpu_percent >= w[1].cpu_percent),
            "cpu list sorted descending"
        );
    }
}
