import { useState, useEffect } from "react";
import type { Session } from "../../types/bindings";

interface SessionTimerProps {
  session: Session | null;
}

function formatElapsed(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function SessionTimer({ session }: SessionTimerProps) {
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (!session) {
      setElapsed(0);
      return;
    }

    const startedAt = new Date(session.started_at).getTime();
    const update = () => {
      setElapsed(Math.floor((Date.now() - startedAt) / 1000));
    };

    update();
    const id = setInterval(update, 1000);
    return () => clearInterval(id);
  }, [session]);

  if (!session) return null;

  return (
    <div className="session-timer">
      <span className="session-timer__icon">🟢</span>
      <span className="session-timer__time">{formatElapsed(elapsed)}</span>
    </div>
  );
}
