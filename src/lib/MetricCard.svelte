<script lang="ts">
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import Gauge from "./Gauge.svelte";
  import Sparkline from "./Sparkline.svelte";
  import type { UsageMetric } from "./stores/metrics";

  export let label: string;
  export let icon: "cpu" | "gpu" | "ram" | "disk";
  export let metric: UsageMetric;
  export let history: (number | null)[] = [];
  export let showBytes = false;

  // Same duration/easing as the gauge so number and arc move together.
  const display = tweened(0, { duration: 600, easing: cubicOut });
  $: display.set(metric.percent ?? 0);

  $: level =
    metric.percent === null
      ? "unavailable"
      : metric.percent >= 99
        ? "very-high"
        : metric.percent >= 85
          ? "high"
          : metric.percent >= 60
            ? "medium"
            : "low";

  function fmtBytes(n: number | null): string {
    if (n === null) return "--";
    const gb = n / 1024 ** 3;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(n / 1024 ** 2).toFixed(0)} MB`;
  }

  const ICON_PATHS: Record<string, string> = {
    cpu: "M9 2v2M15 2v2M9 20v2M15 20v2M2 9h2M2 15h2M20 9h2M20 15h2M6 6h12v12H6zM9.5 9.5h5v5h-5z",
    gpu: "M2 7h20v10H4a2 2 0 0 1-2-2V7zM2 7V5m5 12v2m10-2v2M14.5 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM18 9.5h1.5",
    ram: "M3 7h18v8H3zM3 15v2m4-2v2m4-2v2m4-2v2m4-2v2M6.5 10v2m3.5-2v2m3.5-2v2m3.5-2v2",
    disk: "M12 3c-4.4 0-8 1.3-8 3v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6c0-1.7-3.6-3-8-3zM4 6c0 1.7 3.6 3 8 3s8-1.3 8-3M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3",
  };
</script>

<article class="card" data-level={level}>
  <header class="card-header">
    <span class="card-id">
      <svg class="card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d={ICON_PATHS[icon]} />
      </svg>
      <span class="card-label">{label}</span>
    </span>
    {#if metric.temp_celsius !== null}
      <span class="pill">{Math.round(metric.temp_celsius)}°C</span>
    {/if}
  </header>

  <div class="gauge-wrap">
    <Gauge value={metric.percent} />
    <div class="gauge-center">
      {#if metric.percent !== null}
        <span class="value">{Math.round($display)}<small>%</small></span>
        {#if level === "very-high"}
          <span class="status critical">▲ critical</span>
        {:else if level === "high"}
          <span class="status high">▲ high</span>
        {/if}
      {:else}
        <span class="value muted">--</span>
        <span class="status none">no sensor</span>
      {/if}
    </div>
  </div>

  <footer class="card-foot">
    {#if showBytes && metric.used_bytes !== null && metric.total_bytes !== null}
      <span class="bytes">{fmtBytes(metric.used_bytes)} <em>/ {fmtBytes(metric.total_bytes)}</em></span>
    {/if}
    <Sparkline values={history} />
  </footer>
</article>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 12px 10px;
    background: var(--surface);
    backdrop-filter: var(--glass);
    -webkit-backdrop-filter: var(--glass);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    transition: border-color 400ms ease, box-shadow 400ms ease;
  }

  .card[data-level="very-high"] {
    border-color: var(--danger-soft);
    box-shadow: var(--shadow), var(--glow-critical);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 20px;
  }

  .card-id {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 700;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .card-icon {
    width: 15px;
    height: 15px;
    opacity: 0.85;
  }

  .gauge-wrap {
    position: relative;
    display: grid;
    place-items: center;
    flex: 1;
  }

  .gauge-center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1px;
  }

  .value {
    font-size: 23px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.01em;
  }

  .value small {
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    margin-left: 1px;
  }

  .value.muted {
    color: var(--muted);
  }

  .status {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .status.critical {
    color: var(--danger);
  }

  .status.high {
    color: var(--warn);
  }

  .status.none {
    color: var(--muted);
    font-style: italic;
    text-transform: none;
    letter-spacing: 0.02em;
  }

  .card-foot {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-height: 44px;
    justify-content: flex-end;
  }

  .bytes {
    font-family: var(--font-mono);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }

  .bytes em {
    font-style: normal;
    color: var(--muted);
  }

  .pill {
    font-family: var(--font-mono);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: var(--muted);
    background: var(--surface-strong);
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 1px 7px;
    white-space: nowrap;
  }
</style>
