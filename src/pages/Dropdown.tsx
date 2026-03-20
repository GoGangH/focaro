import { useEffect, useState } from "react";
import type { Session } from "../types/bindings";
import { useSession } from "../hooks/useSession";
import { SessionTimer } from "../components/Dropdown/SessionTimer";
import { SessionControls } from "../components/Dropdown/SessionControls";
import { getIncompleteSession, resumeSession, discardIncompleteSession, openDashboard } from "../services/session";

export function Dropdown() {
  const { session, isLoading, startSession, endSession } = useSession();
  const [incompleteSession, setIncompleteSession] = useState<Session | null>(null);
  const [recoveryChecked, setRecoveryChecked] = useState(false);

  // 앱 시작 시 미완료 세션 확인
  useEffect(() => {
    if (recoveryChecked) return;
    setRecoveryChecked(true);

    getIncompleteSession().then((s) => {
      if (s) setIncompleteSession(s);
    });
  }, [recoveryChecked]);

  const handleResume = async () => {
    if (!incompleteSession) return;
    await resumeSession(incompleteSession.id);
    setIncompleteSession(null);
  };

  const handleDiscard = async () => {
    if (!incompleteSession) return;
    await discardIncompleteSession(incompleteSession.id);
    setIncompleteSession(null);
  };

  // 미완료 세션 복구 팝업
  if (incompleteSession) {
    return (
      <div className="dropdown dropdown--recovery">
        <p className="recovery__title">이전 세션이 종료되지 않았습니다</p>
        <p className="recovery__subtitle">이어서 진행할까요?</p>
        <div className="recovery__actions">
          <button onClick={handleResume} className="session-btn session-btn--start">
            이어가기
          </button>
          <button onClick={handleDiscard} className="session-btn session-btn--end">
            종료
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="dropdown">
      <SessionTimer session={session} />
      <SessionControls
        session={session}
        isLoading={isLoading}
        onStart={startSession}
        onEnd={endSession}
      />
      <button onClick={openDashboard} className="dashboard-btn">
        Dashboard 열기
      </button>
    </div>
  );
}
