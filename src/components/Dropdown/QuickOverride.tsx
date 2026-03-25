import { useState } from "react";
import { addTitleRule } from "../../services/onboarding";

interface QuickOverrideProps {
  domain: string | null;
  title: string | null;
  currentCategory: string;
  onClose: () => void;
}

const CATEGORIES = [
  { value: "Focus", label: "집중", color: "#30d158" },
  { value: "Neutral", label: "기타", color: "#636366" },
  { value: "Distraction", label: "방해", color: "#ff453a" },
];

export function QuickOverride({ domain, title, currentCategory, onClose }: QuickOverrideProps) {
  const [mode, setMode] = useState<"once" | "always">("once");
  const [loading, setLoading] = useState(false);

  const handleApply = async (category: string) => {
    if (loading) return;
    setLoading(true);
    try {
      if (mode === "always" && domain && title) {
        // 제목 키워드 기반 영구 규칙 생성
        // 타이틀의 첫 번째 의미있는 단어를 keyword로 사용
        const keyword = extractKeyword(title);
        if (keyword) {
          await addTitleRule(domain, keyword, category);
        }
      }
      onClose();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="quick-override">
      <div className="quick-override__header">
        <span className="quick-override__domain">{domain ?? "현재 사이트"}</span>
        <button className="quick-override__close" onClick={onClose}>×</button>
      </div>
      {title && (
        <div className="quick-override__title">{title}</div>
      )}
      <div className="quick-override__mode">
        <button
          className={`quick-override__mode-btn${mode === "once" ? " active" : ""}`}
          onClick={() => setMode("once")}
        >
          이번만
        </button>
        <button
          className={`quick-override__mode-btn${mode === "always" ? " active" : ""}`}
          onClick={() => setMode("always")}
        >
          항상 이렇게
        </button>
      </div>
      <div className="quick-override__cats">
        {CATEGORIES.filter((c) => c.value !== currentCategory).map((cat) => (
          <button
            key={cat.value}
            className="quick-override__cat-btn"
            style={{ borderColor: cat.color, color: cat.color }}
            onClick={() => handleApply(cat.value)}
            disabled={loading}
          >
            {`${cat.label}로 변경`}
          </button>
        ))}
      </div>
    </div>
  );
}

function extractKeyword(title: string): string {
  // 타이틀에서 3글자 이상의 단어를 keyword로 추출
  const words = title.split(/\s+/).filter((w) => w.length >= 3);
  return words[0]?.toLowerCase() ?? "";
}
