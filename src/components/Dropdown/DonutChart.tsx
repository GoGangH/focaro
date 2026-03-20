interface DonutChartProps {
  focus: number;
  neutral: number;
  distraction: number;
  size?: number;
}

export function DonutChart({ focus, neutral, distraction, size = 80 }: DonutChartProps) {
  const total = focus + neutral + distraction;
  const radius = 28;
  const circumference = 2 * Math.PI * radius;
  const cx = size / 2;
  const cy = size / 2;

  const focusPct = total > 0 ? focus / total : 0;
  const neutralPct = total > 0 ? neutral / total : 0;
  const distractionPct = total > 0 ? distraction / total : 0;

  // Segments drawn clockwise from top (-90deg)
  const focusDash = circumference * focusPct;
  const neutralDash = circumference * neutralPct;
  const distractionDash = circumference * distractionPct;

  const focusOffset = 0;
  const neutralOffset = -(focusDash);
  const distractionOffset = -(focusDash + neutralDash);

  const focusPercent = Math.round(focusPct * 100);

  return (
    <div style={{ position: "relative", width: size, height: size, flexShrink: 0 }}>
      <svg width={size} height={size} style={{ transform: "rotate(-90deg)" }}>
        {/* Background */}
        <circle cx={cx} cy={cy} r={radius} fill="none" stroke="#2c2c2e" strokeWidth={8} />

        {total === 0 ? (
          <circle cx={cx} cy={cy} r={radius} fill="none" stroke="#3a3a3c" strokeWidth={8} />
        ) : (
          <>
            {focusDash > 0 && (
              <circle
                cx={cx} cy={cy} r={radius}
                fill="none" stroke="#30d158" strokeWidth={8}
                strokeDasharray={`${focusDash} ${circumference}`}
                strokeDashoffset={focusOffset}
                strokeLinecap="butt"
              />
            )}
            {neutralDash > 0 && (
              <circle
                cx={cx} cy={cy} r={radius}
                fill="none" stroke="#636366" strokeWidth={8}
                strokeDasharray={`${neutralDash} ${circumference}`}
                strokeDashoffset={neutralOffset}
                strokeLinecap="butt"
              />
            )}
            {distractionDash > 0 && (
              <circle
                cx={cx} cy={cy} r={radius}
                fill="none" stroke="#ff453a" strokeWidth={8}
                strokeDasharray={`${distractionDash} ${circumference}`}
                strokeDashoffset={distractionOffset}
                strokeLinecap="butt"
              />
            )}
          </>
        )}
      </svg>

      {/* Center label */}
      <div style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 0,
      }}>
        <span style={{ fontSize: 15, fontWeight: 700, color: "#30d158", lineHeight: 1 }}>
          {focusPercent}%
        </span>
        <span style={{ fontSize: 9, color: "#8e8e93", lineHeight: 1.4 }}>focus</span>
      </div>
    </div>
  );
}
