<script lang="ts">
  /** Percent history, oldest first; nulls (sensor gaps) break the line. */
  export let values: (number | null)[] = [];

  const W = 120;
  const H = 28;
  const PAD = 2;

  $: points = values.map((v, i) => ({
    x: values.length > 1 ? (i / (values.length - 1)) * W : W,
    y: v === null ? null : H - PAD - (Math.min(Math.max(v, 0), 100) / 100) * (H - PAD * 2),
  }));

  // One path per contiguous non-null run.
  $: linePath = points
    .map((p, i) => {
      if (p.y === null) return "";
      const prev = i > 0 ? points[i - 1] : null;
      const cmd = prev && prev.y !== null ? "L" : "M";
      return `${cmd} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`;
    })
    .join(" ");

  // Area wash under the last contiguous run, closed to the baseline.
  $: areaPath = (() => {
    let start = points.length - 1;
    while (start > 0 && points[start - 1].y !== null) start -= 1;
    const run = points.slice(start).filter((p) => p.y !== null);
    if (run.length < 2) return "";
    const line = run.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(1)} ${p.y!.toFixed(1)}`).join(" ");
    return `${line} L ${run[run.length - 1].x.toFixed(1)} ${H} L ${run[0].x.toFixed(1)} ${H} Z`;
  })();

  $: hasData = points.some((p) => p.y !== null);
</script>

{#if hasData}
  <svg class="spark" viewBox="0 0 {W} {H}" preserveAspectRatio="none" aria-hidden="true">
    {#if areaPath}
      <path class="spark-area" d={areaPath} />
    {/if}
    <path class="spark-line" d={linePath} fill="none" />
  </svg>
{/if}

<style>
  .spark {
    display: block;
    width: 100%;
    height: 28px;
    opacity: 0.9;
  }

  .spark-line {
    stroke: var(--ok);
    stroke-width: 1.5;
    stroke-linejoin: round;
    stroke-linecap: round;
    vector-effect: non-scaling-stroke;
  }

  .spark-area {
    fill: var(--ok);
    opacity: 0.1;
  }
</style>
