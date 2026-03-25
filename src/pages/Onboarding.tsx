import { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { applyProfessionRules, completeOnboarding, type Profession } from "../services/onboarding";

type Step = "welcome" | "profession" | "done";

const PROFESSIONS: { value: Profession; label: string; description: string }[] = [
  { value: "developer", label: "개발자", description: "코딩, 터미널, GitHub 등" },
  { value: "designer", label: "디자이너", description: "Figma, Dribbble, Behance 등" },
  { value: "marketer", label: "마케터", description: "Analytics, Ads, HubSpot 등" },
  { value: "student", label: "학생", description: "Coursera, Khan Academy, Wikipedia 등" },
  { value: "other", label: "기타 / 직접 설정", description: "기본 규칙만 적용됩니다" },
];

export function Onboarding() {
  const [step, setStep] = useState<Step>("welcome");
  const [selected, setSelected] = useState<Profession | null>(null);
  const [loading, setLoading] = useState(false);

  const handleProfessionNext = async () => {
    if (!selected) return;
    setLoading(true);
    try {
      await applyProfessionRules(selected);
      setStep("done");
    } finally {
      setLoading(false);
    }
  };

  const handleSkip = async () => {
    setLoading(true);
    try {
      await completeOnboarding();
      await getCurrentWindow().close();
    } finally {
      setLoading(false);
    }
  };

  const handleFinish = async () => {
    setLoading(true);
    try {
      await completeOnboarding();
      await getCurrentWindow().close();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="onboarding">
      {step === "welcome" && (
        <div className="onboarding__step">
          <div className="onboarding__icon">🎯</div>
          <h1 className="onboarding__title">focaro에 오신 것을 환영합니다</h1>
          <p className="onboarding__desc">
            focaro는 컴퓨터 활동을 추적하여 집중 시간을 분석하는 메뉴바 앱입니다.
          </p>
          <ul className="onboarding__features">
            <li>✅ 앱 및 브라우저 활동 자동 추적</li>
            <li>✅ Focus / Neutral / Distraction 분류</li>
            <li>✅ 세션 단위 집중도 분석</li>
            <li>✅ Reference URL 저장</li>
          </ul>
          <div className="onboarding__actions">
            <button
              className="onboarding__btn onboarding__btn--primary"
              onClick={() => setStep("profession")}
            >
              시작하기
            </button>
            <button
              className="onboarding__btn onboarding__btn--ghost"
              onClick={handleSkip}
              disabled={loading}
            >
              건너뛰기
            </button>
          </div>
        </div>
      )}

      {step === "profession" && (
        <div className="onboarding__step">
          <h2 className="onboarding__title">직업을 선택해주세요</h2>
          <p className="onboarding__desc">
            선택에 맞는 앱/사이트가 자동으로 Focus로 분류됩니다.
          </p>
          <div className="onboarding__professions">
            {PROFESSIONS.map((p) => (
              <button
                key={p.value}
                className={`onboarding__profession${selected === p.value ? " onboarding__profession--selected" : ""}`}
                onClick={() => setSelected(p.value)}
              >
                <span className="onboarding__profession-label">{p.label}</span>
                <span className="onboarding__profession-desc">{p.description}</span>
              </button>
            ))}
          </div>
          <div className="onboarding__actions">
            <button
              className="onboarding__btn onboarding__btn--primary"
              onClick={handleProfessionNext}
              disabled={!selected || loading}
            >
              {loading ? "적용 중..." : "다음"}
            </button>
            <button
              className="onboarding__btn onboarding__btn--ghost"
              onClick={handleSkip}
              disabled={loading}
            >
              건너뛰기
            </button>
          </div>
        </div>
      )}

      {step === "done" && (
        <div className="onboarding__step">
          <div className="onboarding__icon">🟢</div>
          <h2 className="onboarding__title">설정 완료!</h2>
          <p className="onboarding__desc">
            메뉴바 아이콘을 클릭하여 세션을 시작할 수 있습니다.
            <br />
            설정 &gt; 분류 규칙에서 언제든지 수정할 수 있습니다.
          </p>
          <div className="onboarding__actions">
            <button
              className="onboarding__btn onboarding__btn--primary"
              onClick={handleFinish}
              disabled={loading}
            >
              {loading ? "저장 중..." : "시작하기"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
