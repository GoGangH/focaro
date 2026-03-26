import { useEffect, useState } from "react";
import { getTrend, type TrendPoint } from "../../services/stats";

function formatDate(dateStr: string): string {
  const d = new Date(dateStr);
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

export function TrendChart() {
  const [points, setPoints] = useState<TrendPoint[]>([]);

  useEffect(() => {
    getTrend(30).then(setPoints);
  }, []);

  if (points.length === 0) {
    return <p className="dash-empty">최근 30일 데이터가 없습니다</p>;
  }

  const W = 560;
  const H = 180;
  const PAD = { top: 16, right: 16, bottom: 32, left: 36 };
  const chartW = W - PAD.left - PAD.right;
  const chartH = H - PAD.top - PAD.bottom;

  const maxPct = 100;
  const xStep = points.length > 1 ? chartW / (points.length - 1) : chartW;

  const toX = (i: number) => PAD.left + i * xStep;
  const toY = (pct: number) => PAD.top + chartH - (pct / maxPct) * chartH;

  const polyline = points
    .map((p, i) => `${toX(i)},${toY(p.focus_pct)}`)
    .join(" ");

  // area fill path
  const areaPath = [
    `M ${toX(0)},${toY(points[0].focus_pct)}`,
    ...points.slice(1).map((p, i) => `L ${toX(i + 1)},${toY(p.focus_pct)}`),
    `L ${toX(points.length - 1)},${PAD.top + chartH}`,
    `L ${toX(0)},${PAD.top + chartH}`,
    "Z",
  ].join(" ");

  // Y-axis ticks
  const yTicks = [0, 25, 50, 75, 100];

  // X-axis labels: show first, middle, last
  const xLabelIndices = new Set([
    0,
    Math.floor((points.length - 1) / 2),
    points.length - 1,
  ]);

  const avgPct =
    points.reduce((s, p) => s + p.focus_pct, 0) / points.length;

  return (
    <div className="trend-chart">
      <div className="trend-chart__header">
        <span className="trend-chart__title">최근 30일 Focus 추이</span>
        <span className="trend-chart__avg">평균 {avgPct.toFixed(0)}%</span>
      </div>

      <svg
        viewBox={`0 0 ${W} ${H}`}
        width="100%"
        className="trend-chart__svg"
      >
        <defs>
          <linearGradient id="trend-fill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#0a84ff" stopOpacity="0.3" />
            <stop offset="100%" stopColor="#0a84ff" stopOpacity="0" />
          </linearGradient>
        </defs>

        {/* Y-axis grid lines */}
        {yTicks.map((tick) => (
          <g key={tick}>
            <line
              x1={PAD.left}
              y1={toY(tick)}
              x2={W - PAD.right}
              y2={toY(tick)}
              stroke="#333"
              strokeWidth={0.5}
              strokeDasharray="3,3"
            />
            <text
              x={PAD.left - 6}
              y={toY(tick) + 4}
              textAnchor="end"
              fontSize={9}
              fill="#888"
            >
              {tick}%
            </text>
          </g>
        ))}

        {/* Area fill */}
        <path d={areaPath} fill="url(#trend-fill)" />

        {/* Line */}
        <polyline
          points={polyline}
          fill="none"
          stroke="#0a84ff"
          strokeWidth={2}
          strokeLinejoin="round"
          strokeLinecap="round"
        />

        {/* Data points */}
        {points.map((p, i) => (
          <circle
            key={p.date}
            cx={toX(i)}
            cy={toY(p.focus_pct)}
            r={3}
            fill="#0a84ff"
          >
            <title>{`${p.date}: ${p.focus_pct.toFixed(0)}%`}</title>
          </circle>
        ))}

        {/* X-axis labels */}
        {points.map((p, i) =>
          xLabelIndices.has(i) ? (
            <text
              key={p.date}
              x={toX(i)}
              y={H - 4}
              textAnchor="middle"
              fontSize={9}
              fill="#888"
            >
              {formatDate(p.date)}
            </text>
          ) : null
        )}
      </svg>
    </div>
  );
}
