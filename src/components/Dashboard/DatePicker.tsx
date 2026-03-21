import { useState, useRef, useEffect, useCallback } from "react";
import { createPortal } from "react-dom";

interface Props {
  value: string;          // "YYYY-MM-DD"
  max: string;            // "YYYY-MM-DD" — 이 날짜 이후 선택 불가
  onChange: (date: string) => void;
}

function toYM(dateStr: string) {
  const [y, m] = dateStr.split("-").map(Number);
  return { year: y, month: m };
}

function daysInMonth(year: number, month: number) {
  return new Date(year, month, 0).getDate();
}

function firstDayOfMonth(year: number, month: number) {
  return (new Date(year, month - 1, 1).getDay() + 6) % 7; // Mon=0
}

function toDateStr(year: number, month: number, day: number) {
  return `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}

function shiftDate(dateStr: string, days: number): string {
  const d = new Date(dateStr);
  d.setDate(d.getDate() + days);
  return d.toISOString().split("T")[0];
}

function formatLabel(value: string, max: string): string {
  const yesterday = shiftDate(max, -1);
  if (value === max) return "오늘";
  if (value === yesterday) return "어제";
  const d = new Date(value);
  return d.toLocaleDateString("ko-KR", { month: "long", day: "numeric", weekday: "short" });
}

const WEEK = ["월", "화", "수", "목", "금", "토", "일"];
const POPUP_W = 240;

export function DatePicker({ value, max, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState({ top: 0, left: 0 });
  const [{ year, month }, setYM] = useState(() => toYM(value));
  const triggerRef = useRef<HTMLButtonElement>(null);

  useEffect(() => { setYM(toYM(value)); }, [value]);

  // 팝업 위치 계산 — 트리거 버튼 기준 fixed 좌표
  const calcPos = useCallback(() => {
    if (!triggerRef.current) return;
    const r = triggerRef.current.getBoundingClientRect();
    // 대시보드 헤더 우측 패딩(24px)에 맞춰 팝업 오른쪽 끝 고정
    const left = window.innerWidth - POPUP_W - 24;
    setPos({ top: r.bottom + 6, left });
  }, []);

  const handleOpen = () => {
    calcPos();
    setOpen((v) => !v);
  };

  // 외부 클릭 시 닫기
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      const target = e.target as Node;
      if (triggerRef.current?.contains(target)) return;
      // 팝업 내부 클릭은 팝업 자체에서 막으므로 여기선 닫기
      setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const { year: maxY, month: maxM } = toYM(max);
  const maxDay = Number(max.split("-")[2]);
  const isMaxMonth = year === maxY && month === maxM;

  function prevMonth() {
    setYM(({ year: y, month: m }) =>
      m === 1 ? { year: y - 1, month: 12 } : { year: y, month: m - 1 }
    );
  }

  function nextMonth() {
    if (isMaxMonth) return;
    setYM(({ year: y, month: m }) =>
      m === 12 ? { year: y + 1, month: 1 } : { year: y, month: m + 1 }
    );
  }

  function isDisabled(day: number) {
    if (year > maxY) return true;
    if (year === maxY && month > maxM) return true;
    if (year === maxY && month === maxM && day > maxDay) return true;
    return false;
  }

  const totalDays = daysInMonth(year, month);
  const startOffset = firstDayOfMonth(year, month);
  const cells: (number | null)[] = [
    ...Array(startOffset).fill(null),
    ...Array.from({ length: totalDays }, (_, i) => i + 1),
  ];
  while (cells.length % 7 !== 0) cells.push(null);

  const popup = open
    ? createPortal(
        <div
          className="datepicker-popup"
          style={{ top: pos.top, left: pos.left }}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <div className="datepicker-header">
            <button className="datepicker-nav" onClick={prevMonth}>‹</button>
            <span className="datepicker-month">{year}년 {month}월</span>
            <button className="datepicker-nav" onClick={nextMonth} disabled={isMaxMonth}>›</button>
          </div>

          <div className="datepicker-grid">
            {WEEK.map((w) => (
              <span key={w} className="datepicker-weekday">{w}</span>
            ))}
            {cells.map((day, idx) => {
              if (!day) return <span key={`e-${idx}`} />;
              const disabled = isDisabled(day);
              const selected = value === toDateStr(year, month, day);
              const today = max === toDateStr(year, month, day);
              return (
                <button
                  key={day}
                  className={[
                    "datepicker-day",
                    selected ? "datepicker-day--selected" : "",
                    today && !selected ? "datepicker-day--today" : "",
                    disabled ? "datepicker-day--disabled" : "",
                  ].filter(Boolean).join(" ")}
                  disabled={disabled}
                  onClick={() => {
                    onChange(toDateStr(year, month, day));
                    setOpen(false);
                  }}
                >
                  {day}
                </button>
              );
            })}
          </div>
        </div>,
        document.body
      )
    : null;

  return (
    <>
      <button
        ref={triggerRef}
        className="dash-date-label datepicker-trigger"
        onClick={handleOpen}
        type="button"
      >
        {formatLabel(value, max)}
      </button>
      {popup}
    </>
  );
}
