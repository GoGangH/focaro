import { useState, useEffect } from "react";
import type { TitleRule } from "../../types/bindings";
import { getTitleRules, deleteTitleRule } from "../../services/settings";

const CATEGORY_COLORS: Record<string, string> = {
  Focus: "focus",
  Neutral: "neutral",
  Distraction: "distraction",
};

export function TitleRuleSettings() {
  const [rules, setRules] = useState<TitleRule[]>([]);

  useEffect(() => {
    getTitleRules().then(setRules).catch(() => {});
  }, []);

  async function handleDelete(id: number) {
    await deleteTitleRule(id);
    setRules((prev) => prev.filter((r) => r.id !== id));
  }

  return (
    <section className="settings-section">
      <h3 className="settings-section-title">타이틀 규칙</h3>
      <p className="settings-desc">
        Quick Override로 저장된 페이지 제목 키워드 기반 분류 규칙입니다.
      </p>

      <div className="settings-rules-list">
        {rules.length === 0 && (
          <p className="settings-empty">등록된 타이틀 규칙이 없습니다.</p>
        )}
        {rules.map((rule) => (
          <div key={rule.id} className="settings-rule-row">
            <span className="settings-rule-domain">{rule.domain}</span>
            <span className="settings-title-rule-keyword">"{rule.keyword}"</span>
            <span
              className={`settings-rule-badge settings-rule-badge--${CATEGORY_COLORS[rule.category] ?? "neutral"}`}
            >
              {rule.category}
            </span>
            <button
              className="settings-btn-sm settings-btn-sm--danger"
              onClick={() => handleDelete(rule.id)}
            >
              삭제
            </button>
          </div>
        ))}
      </div>
    </section>
  );
}
