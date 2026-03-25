import { useEffect, useState } from "react";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";

export function AutoLaunchSettings() {
  const [enabled, setEnabled] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    isEnabled().then((v: boolean) => {
      setEnabled(v);
      setLoading(false);
    });
  }, []);

  const handleToggle = async () => {
    setLoading(true);
    try {
      if (enabled) {
        await disable();
        setEnabled(false);
      } else {
        await enable();
        setEnabled(true);
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="settings-section">
      <h3 className="settings-section__title">시작 프로그램</h3>
      <label className="settings-toggle">
        <span className="settings-toggle__label">로그인 시 자동 실행</span>
        <input
          type="checkbox"
          className="settings-toggle__input"
          checked={enabled}
          onChange={handleToggle}
          disabled={loading}
        />
        <span className="settings-toggle__slider" />
      </label>
    </div>
  );
}
