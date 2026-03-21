import type { DomainSummary, Classification } from "../../types/bindings";

function classColor(cls: Classification | string): string {
  if (cls === "Focus") return "#30d158";
  if (cls === "Distraction") return "#ff453a";
  return "#636366";
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}초`;
  const m = Math.floor(secs / 60);
  if (m < 60) return `${m}분`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm > 0 ? `${h}h ${String(rm).padStart(2, "0")}m` : `${h}h 00m`;
}

interface Props {
  sites: DomainSummary[];
}

export function TopSites({ sites }: Props) {
  if (sites.length === 0) {
    return <p className="dash-empty">사이트 없음</p>;
  }

  const maxSecs = sites[0]?.total_secs ?? 1;

  return (
    <div className="top-sites">
      {sites.map((site) => (
        <div key={site.domain} className="site-row">
          <div className="site-row__header">
            <span
              className="site-dot"
              style={{ background: classColor(site.classification) }}
            />
            <span className="site-domain">{site.domain}</span>
            <span className="site-duration">{formatDuration(site.total_secs)}</span>
          </div>
          <div className="site-bar-bg">
            <div
              className="site-bar"
              style={{
                width: `${(site.total_secs / maxSecs) * 100}%`,
                background: classColor(site.classification),
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}
