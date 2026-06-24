/** Cumulative-solved line chart: indigo line over a soft area fill. */
export function ProgressChart({ series }: { series: number[] }) {
  const W = 400;
  const H = 160;
  if (series.length < 2) return null;

  const max = Math.max(...series, 1);
  const pts = series.map((v, i) => {
    const x = (i / (series.length - 1)) * W;
    const y = H - 8 - (v / max) * (H - 20);
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });
  const line = `M${pts.join(" L")}`;
  const area = `${line} L${W},${H} L0,${H} Z`;

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      preserveAspectRatio="none"
      className="block h-40 w-full"
      aria-label="Cumulative problems solved over time"
    >
      {[40, 80, 120].map((y) => (
        <line
          key={y}
          x1={0}
          y1={y}
          x2={W}
          y2={y}
          className="stroke-border"
          strokeWidth={1}
          strokeDasharray="3 4"
        />
      ))}
      <path d={area} className="fill-primary/15" stroke="none" />
      <path
        d={line}
        fill="none"
        className="stroke-primary"
        strokeWidth={2.5}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
