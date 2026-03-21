import type { Session } from "../../types/bindings";

interface SessionControlsProps {
  session: Session | null;
  isLoading: boolean;
  onStart: () => Promise<void>;
  onEnd: () => Promise<void>;
}

export function SessionControls({ session, isLoading, onStart, onEnd }: SessionControlsProps) {
  if (session) {
    return (
      <button
        onClick={onEnd}
        disabled={isLoading}
        className="session-btn session-btn--end"
      >
        {isLoading ? "종료 중..." : "세션 종료"}
      </button>
    );
  }

  return (
    <button
      onClick={onStart}
      disabled={isLoading}
      className="session-btn session-btn--start"
    >
      {isLoading ? "시작 중..." : "세션 시작"}
    </button>
  );
}
