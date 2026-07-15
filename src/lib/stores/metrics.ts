import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";

export interface UsageMetric {
  percent: number | null;
  used_bytes: number | null;
  total_bytes: number | null;
  temp_celsius: number | null;
}

export interface MetricsSnapshot {
  cpu: UsageMetric;
  ram: UsageMetric;
  disk: UsageMetric;
  gpu: UsageMetric;
  timestamp_ms: number;
}

const EMPTY_METRIC: UsageMetric = {
  percent: null,
  used_bytes: null,
  total_bytes: null,
  temp_celsius: null,
};

export const EMPTY_SNAPSHOT: MetricsSnapshot = {
  cpu: EMPTY_METRIC,
  ram: EMPTY_METRIC,
  disk: EMPTY_METRIC,
  gpu: EMPTY_METRIC,
  timestamp_ms: 0,
};

/** How many samples the sparklines keep (~1 minute at the visible poll rate). */
const HISTORY_LENGTH = 60;

export const metrics = writable<MetricsSnapshot>(EMPTY_SNAPSHOT);
export const history = writable<MetricsSnapshot[]>([]);

function push(snapshot: MetricsSnapshot) {
  metrics.set(snapshot);
  history.update((h) => [...h.slice(-(HISTORY_LENGTH - 1)), snapshot]);
}

/**
 * Fetches the current snapshot for an instant first paint, then subscribes to
 * the backend's live event stream. Returns a cleanup function to unsubscribe.
 *
 * Outside Tauri (plain `vite dev` in a browser) there is no backend, so a
 * demo stream stands in — this keeps the UI developable in a browser tab.
 */
export async function startMetricsStream(): Promise<() => void> {
  if (!("__TAURI_INTERNALS__" in window)) {
    return startDemoStream();
  }

  try {
    const initial = await invoke<MetricsSnapshot>("get_current_metrics");
    push(initial);
  } catch (err) {
    console.error("failed to fetch initial metrics", err);
  }

  let unlisten: UnlistenFn | null = await listen<MetricsSnapshot>(
    "metrics://update",
    (event) => push(event.payload),
  );

  return () => {
    unlisten?.();
    unlisten = null;
  };
}

function startDemoStream(): () => void {
  const GIB = 1024 ** 3;
  const walk = { cpu: 34, ram: 62, disk: 71, gpu: 48 };
  const step = (v: number) => Math.min(98, Math.max(2, v + (Math.random() - 0.5) * 9));

  const sample = (): MetricsSnapshot => {
    walk.cpu = step(walk.cpu);
    walk.ram = step(walk.ram);
    walk.gpu = step(walk.gpu);
    return {
      cpu: { percent: walk.cpu, used_bytes: null, total_bytes: null, temp_celsius: 52 + walk.cpu / 4 },
      ram: { percent: walk.ram, used_bytes: (walk.ram / 100) * 16 * GIB, total_bytes: 16 * GIB, temp_celsius: null },
      disk: { percent: walk.disk, used_bytes: (walk.disk / 100) * 512 * GIB, total_bytes: 512 * GIB, temp_celsius: null },
      gpu: { percent: walk.gpu, used_bytes: null, total_bytes: null, temp_celsius: 44 + walk.gpu / 3 },
      timestamp_ms: Date.now(),
    };
  };

  push(sample());
  const id = setInterval(() => push(sample()), 1000);
  return () => clearInterval(id);
}
