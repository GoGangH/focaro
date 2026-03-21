import { useState, useRef, useEffect } from "react";

interface Props {
  value: string;          // "YYYY-MM-DD"
  max: string;            // "YYYY-MM-DD" — 이 날짜 이후 선택 불가
  onChange: (date: string) => void;
}

function toYM(dateStr: string) {
  const [y, m] = dateStr.split("-").map(Number);
  return { year: y, month: m }; // month: 1–12
}

function daysInMonth(year: number, month: number) {
  return new Date(year, month, 0).getDate();
}

function firstDayOfMonth(year: number, month: number) {
  // 0=Sun … 6=Sat → shift to Mon=0
  return (new Date(year, month - 1, 1).getDay() + 6) % 7;
}

function toDateStr(year: number, month: number, day: number) {
  return `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}

const WEEK = ["월", "화", "수", "목", "금", "토", "일"];

export function DatePicker({ value, max, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const [{ year, month }, setYM] = useState(() => toYM(value));
  const ref = useRef<HTMLDivElement>(null);

  // value가 바뀌면 달력 월도 동기화
  useEffect(() => { setYM(toYM(value)); }, [value]);

  // 외부 클릭 시 닫기
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const { year: maxY, month: maxM } = toYM(max);
  const maxDay = Number(max.split("-")[2]);
  const isMaxMonth = year === maxY && month === maxM;

  function prevMonth() {
    if (month === 1) setYM({ year: year - 1, month: 12 });
    else setYM({ year, month: month - 1 });
  }

  function nextMonth() {
    if (isMaxMonth) return;
    if (month === 12) setYM({ year: year + 1, month: 1 });
    else setYM({ year, month: month + 1 });
  }

  function isDisabled(day: number) {
    if (year > maxY) return true;
    if (year === maxY && month > maxM) return true;
    if (year === maxY && month === maxM && day > maxDay) return true;
    return false;
  }

  function isSelected(day: number) {
    return value === toDateStr(year, month, day);
  }

  function isToday(day: number) {
    return max === toDateStr(year, month, day);
  }

  const totalDays = daysInMonth(year, month);
  const startOffset = firstDayOfMonth(year, month);
  const cells: (number | null)[] = [
    ...Array(startOffset).fill(null),
    ...Array.from({ length: totalDays }, (_, i) => i + 1),
  ];
  // 6행 맞추기
  while (cells.length % 7 !== 0) cells.push(null);

  const monthLabel = `${year}년 ${month}월`;

  return (
    <div className="datepicker-root" ref={ref}>
      <button
        className="dash-date-label datepicker-trigger"
        onClick={() => setOpen((v) => !v)}
        type="button"
      >
        {formatLabel(value, max)}
      </button>

      {open && (
        <div className="datepicker-popup">
          {/* 헤더 */}
          <div className="datepicker-header">
            <button className="datepicker-nav" onClick={prevMonth}>‹</button>
            <span className="datepicker-month">{monthLabel}</span>
            <button
              className="datepicker-nav"
              onClick={nextMonth}
              disabled={isMaxMonth}
            >
              ›
            </button>
          </div>

          {/* 요일 행 */}
          <div className="datepicker-grid">
            {WEEK.map((w) => (
              <span key={w} className="datepicker-weekday">{w}</span>
            ))}

            {/* 날짜 셀 */}
            {cells.map((day, idx) => {
              if (!day) return <span key={`e-${idx}`} />;
              const disabled = isDisabled(day);
              const selected = isSelected(day);
              const today = isToday(day);
              return (
                <button
                  key={day}
                  className={[
                    "datepicker-day",
                    selected ? "datepicker-day--selected" : "",
                    today && !selected ? "datepicker-day--today" : "",
                    disabled ? "datepicker-day--disabled" : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
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
        </div>
      )}
    </div>
  );
}

function formatLabel(value: string, max: string): string {
  const yesterday = shiftDate(max, -1);
  if (value === max) return "오늘";
  if (value === yesterday) return "어제";
  const d = new Date(value);
  return d.toLocaleDateString("ko-KR", { month: "long", day: "numeric", weekday: "short" });
}

function shiftDate(dateStr: string, days: number): string {
  const d = new Date(dateStr);
  d.setDate(d.getDate() + days);
  return d.toISOString().split("T")[0];
}
