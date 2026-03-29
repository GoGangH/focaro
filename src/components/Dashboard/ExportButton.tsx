import { useState } from "react";
import { exportData, type ExportFormat } from "../../services/export";
import { openPath } from "@tauri-apps/plugin-opener";

function todayStr(): string {
  return new Date().toISOString().split("T")[0];
}

function thirtyDaysAgoStr(): string {
  const d = new Date();
  d.setDate(d.getDate() - 30);
  return d.toISOString().split("T")[0];
}

export function ExportButton() {
  const [open, setOpen] = useState(false);
  const [startDate, setStartDate] = useState(thirtyDaysAgoStr);
  const [endDate, setEndDate] = useState(todayStr);
  const [format, setFormat] = useState<ExportFormat>("csv");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [savedPath, setSavedPath] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  async function handleExport() {
    setStatus("loading");
    setErrorMsg("");
    try {
      const path = await exportData(startDate, endDate, format);
      setSavedPath(path);
      setStatus("success");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setErrorMsg(msg);
      setStatus("error");
    }
  }

  function handleReveal() {
    // Open the folder containing the file
    const dir = savedPath.substring(0, savedPath.lastIndexOf("/"));
    openPath(dir).catch(() => openPath(savedPath));
  }

  if (!open) {
    return (
      <button className="export-trigger" onClick={() => { setOpen(true); setStatus("idle"); }}>
        ↓ 내보내기
      </button>
    );
  }

  return (
    <div className="export-panel">
      <div className="export-panel__header">
        <span className="export-panel__title">데이터 내보내기</span>
        <button className="export-panel__close" onClick={() => { setOpen(false); setStatus("idle"); }}>
          ✕
        </button>
      </div>

      <div className="export-panel__row">
        <label className="export-panel__label">시작일</label>
        <input
          type="date"
          className="export-panel__input"
          value={startDate}
          max={endDate}
          onChange={(e) => setStartDate(e.target.value)}
        />
      </div>

      <div className="export-panel__row">
        <label className="export-panel__label">종료일</label>
        <input
          type="date"
          className="export-panel__input"
          value={endDate}
          min={startDate}
          max={todayStr()}
          onChange={(e) => setEndDate(e.target.value)}
        />
      </div>

      <div className="export-panel__row">
        <label className="export-panel__label">형식</label>
        <div className="export-panel__formats">
          {(["csv", "json"] as ExportFormat[]).map((f) => (
            <button
              key={f}
              className={`export-format-btn${format === f ? " export-format-btn--active" : ""}`}
              onClick={() => setFormat(f)}
            >
              {f.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      {status === "success" && (
        <div className="export-panel__success">
          <span>저장 완료</span>
          <button className="export-panel__reveal" onClick={handleReveal}>
            폴더 열기
          </button>
        </div>
      )}

      {status === "error" && (
        <p className="export-panel__error">{errorMsg}</p>
      )}

      <button
        className="export-panel__btn"
        onClick={handleExport}
        disabled={status === "loading"}
      >
        {status === "loading" ? "내보내는 중…" : "내보내기"}
      </button>
    </div>
  );
}
