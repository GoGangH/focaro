import { useState, useEffect } from "react";
import type { AppRule } from "../../types/bindings";
import { getAppRules, addAppRule, deleteAppRule } from "../../services/settings";
import { getTrackedApps } from "../../services/activity";

const CATEGORIES = ["Focus", "Neutral", "Distraction"] as const;
type Category = (typeof CATEGORIES)[number];

export function AppRuleSettings() {
  const [rules, setRules] = useState<AppRule[]>([]);
  const [trackedApps, setTrackedApps] = useState<string[]>([]);
  const [selectedApp, setSelectedApp] = useState("");
  const [customApp, setCustomApp] = useState("");
  const [category, setCategory] = useState<Category>("Focus");
  const [adding, setAdding] = useState(false);
  const [useCustom, setUseCustom] = useState(false);

  useEffect(() => {
    getAppRules().then(setRules).catch(() => {});
    getTrackedApps().then(setTrackedApps).catch(() => {});
  }, []);

  const existingAppNames = new Set(rules.map((r) => r.app_name));

  const appNameToAdd = useCustom ? customApp.trim() : selectedApp;

  async function handleAdd() {
    if (!appNameToAdd) return;
    setAdding(true);
    try {
      const rule = await addAppRule(appNameToAdd, category);
      setRules((prev) => {
        const filtered = prev.filter((r) => r.app_name !== rule.app_name);
        return [...filtered, rule].sort((a, b) => a.app_name.localeCompare(b.app_name));
      });
      setSelectedApp("");
      setCustomApp("");
    } finally {
      setAdding(false);
    }
  }

  async function handleDelete(id: number) {
    await deleteAppRule(id);
    setRules((prev) => prev.filter((r) => r.id !== id));
  }

  return (
    <section className="settings-section">
      <h3 className="settings-section-title">앱 분류 규칙</h3>
      <p className="settings-desc">
        Xcode, Slack 등 네이티브 앱을 Focus / Neutral / Distraction으로 분류합니다.
      </p>

      <div className="settings-rules-list">
        {rules.length === 0 && (
          <p className="settings-empty">등록된 앱 규칙이 없습니다.</p>
        )}
        {rules.map((rule) => (
          <div key={rule.id} className="settings-rule-row">
            <span className="settings-rule-domain">{rule.app_name}</span>
            <span
              className={`settings-rule-badge settings-rule-badge--${rule.category.toLowerCase()}`}
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

      <div className="settings-rule-add">
        <div className="app-rule-source-toggle">
          <button
            className={`app-rule-toggle-btn${!useCustom ? " active" : ""}`}
            onClick={() => setUseCustom(false)}
          >
            추적된 앱
          </button>
          <button
            className={`app-rule-toggle-btn${useCustom ? " active" : ""}`}
            onClick={() => setUseCustom(true)}
          >
            직접 입력
          </button>
        </div>

        {useCustom ? (
          <input
            className="settings-input"
            placeholder="앱 이름 입력 (예: Xcode)"
            value={customApp}
            onChange={(e) => setCustomApp(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAdd()}
          />
        ) : (
          <select
            className="settings-select"
            value={selectedApp}
            onChange={(e) => setSelectedApp(e.target.value)}
          >
            <option value="">앱 선택...</option>
            {trackedApps.map((app) => (
              <option
                key={app}
                value={app}
                disabled={existingAppNames.has(app)}
              >
                {app}{existingAppNames.has(app) ? " (규칙 있음)" : ""}
              </option>
            ))}
          </select>
        )}

        <select
          className="settings-select"
          value={category}
          onChange={(e) => setCategory(e.target.value as Category)}
        >
          {CATEGORIES.map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>

        <button
          className="settings-btn-sm"
          onClick={handleAdd}
          disabled={adding || !appNameToAdd}
        >
          추가
        </button>
      </div>
    </section>
  );
}
