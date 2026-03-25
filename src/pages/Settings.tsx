import { useState, useEffect, useCallback } from "react";
import type { AppSettings, ClassificationRule } from "../types/bindings";
import {
  getSettings,
  updateSettings,
  getClassificationRules,
  addClassificationRule,
  deleteClassificationRule,
} from "../services/settings";
import { AutoLaunchSettings } from "../components/Settings/AutoLaunchSettings";

const RETENTION_OPTIONS = [
  { label: "7일", value: 7 },
  { label: "30일", value: 30 },
  { label: "90일", value: 90 },
  { label: "무제한", value: 0 },
];

const CATEGORY_OPTIONS = ["Focus", "Neutral", "Distraction"] as const;

export function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [rules, setRules] = useState<ClassificationRule[]>([]);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [newDomain, setNewDomain] = useState("");
  const [newCategory, setNewCategory] = useState<string>("Focus");
  const [addingRule, setAddingRule] = useState(false);
  const [shortcutEditing, setShortcutEditing] = useState(false);
  const [shortcutInput, setShortcutInput] = useState("");

  const loadData = useCallback(async () => {
    const [s, r] = await Promise.all([getSettings(), getClassificationRules()]);
    setSettings(s);
    setRules(r);
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSave = async () => {
    if (!settings) return;
    setSaving(true);
    try {
      await updateSettings(settings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  const handleShortcutCapture = (e: React.KeyboardEvent<HTMLInputElement>) => {
    e.preventDefault();
    const parts: string[] = [];
    if (e.metaKey) parts.push("CmdOrCtrl");
    if (e.ctrlKey && !e.metaKey) parts.push("CmdOrCtrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    const key = e.key.toUpperCase();
    if (key.length === 1 || ["F1","F2","F3","F4","F5","F6","F7","F8","F9","F10","F11","F12"].includes(key)) {
      parts.push(key);
      const combo = parts.join("+");
      setShortcutInput(combo);
      setSettings((s) => s ? { ...s, shortcut_save_ref: combo } : s);
      setShortcutEditing(false);
    }
  };

  const handleAddRule = async () => {
    if (!newDomain.trim()) return;
    setAddingRule(true);
    try {
      const rule = await addClassificationRule(newDomain.trim(), newCategory);
      setRules((r) => [...r, rule]);
      setNewDomain("");
    } finally {
      setAddingRule(false);
    }
  };

  const handleDeleteRule = async (id: number) => {
    await deleteClassificationRule(id);
    setRules((r) => r.filter((rule) => rule.id !== id));
  };

  if (!settings) return <div className="settings-loading">로딩 중...</div>;

  return (
    <div className="settings-page">
      <h2 className="settings-title">focaro 설정</h2>

      {/* 자동 실행 설정 */}
      <AutoLaunchSettings />

      {/* 단축키 설정 */}
      <section className="settings-section">
        <h3 className="settings-section-title">전역 단축키</h3>
        <p className="settings-desc">Reference 저장 팝업을 여는 단축키입니다.</p>
        <div className="settings-shortcut-row">
          {shortcutEditing ? (
            <input
              className="settings-shortcut-input"
              placeholder="키를 누르세요..."
              value={shortcutInput}
              onKeyDown={handleShortcutCapture}
              onChange={() => {}}
              autoFocus
            />
          ) : (
            <span className="settings-shortcut-badge">
              {settings.shortcut_save_ref}
            </span>
          )}
          <button
            className="settings-btn-sm"
            onClick={() => {
              setShortcutInput(settings.shortcut_save_ref);
              setShortcutEditing((v) => !v);
            }}
          >
            {shortcutEditing ? "취소" : "변경"}
          </button>
        </div>
      </section>

      {/* 보관 기간 */}
      <section className="settings-section">
        <h3 className="settings-section-title">보관 기간</h3>
        <p className="settings-desc">이보다 오래된 활동 데이터는 자동으로 삭제됩니다.</p>
        <div className="settings-radio-group">
          {RETENTION_OPTIONS.map(({ label, value }) => (
            <label key={value} className="settings-radio-label" aria-label={label}>
              <input
                type="radio"
                name="retention"
                value={value}
                checked={settings.retention_days === value}
                onChange={() => setSettings((s) => s ? { ...s, retention_days: value } : s)}
              />
              {label}
            </label>
          ))}
        </div>
      </section>

      {/* 분류 규칙 */}
      <section className="settings-section">
        <h3 className="settings-section-title">분류 규칙</h3>
        <p className="settings-desc">도메인에 따라 Focus / Neutral / Distraction을 지정합니다.</p>

        <div className="settings-rules-list">
          {rules.map((rule) => (
            <div key={rule.id} className="settings-rule-row">
              <span className="settings-rule-domain">{rule.domain}</span>
              <span className={`settings-rule-badge settings-rule-badge--${rule.category.toLowerCase()}`}>
                {rule.category}
              </span>
              <button
                className="settings-btn-sm settings-btn-sm--danger"
                onClick={() => handleDeleteRule(rule.id)}
              >
                삭제
              </button>
            </div>
          ))}
        </div>

        <div className="settings-rule-add">
          <input
            className="settings-input"
            placeholder="도메인 입력 (예: example.com)"
            value={newDomain}
            onChange={(e) => setNewDomain(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddRule()}
          />
          <select
            className="settings-select"
            value={newCategory}
            onChange={(e) => setNewCategory(e.target.value)}
          >
            {CATEGORY_OPTIONS.map((c) => (
              <option key={c} value={c}>{c}</option>
            ))}
          </select>
          <button
            className="settings-btn-sm"
            onClick={handleAddRule}
            disabled={addingRule || !newDomain.trim()}
          >
            추가
          </button>
        </div>
      </section>

      {/* 저장 버튼 */}
      <div className="settings-footer">
        <button
          className="settings-save-btn"
          onClick={handleSave}
          disabled={saving}
        >
          {saved ? "저장됨 ✓" : saving ? "저장 중..." : "저장"}
        </button>
      </div>
    </div>
  );
}
