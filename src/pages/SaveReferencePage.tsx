import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { saveReference } from "../services/reference";
import { getCurrentUrl, getCurrentTitle } from "../services/session";

export function SaveReferencePage() {
  const [url, setUrl] = useState<string>("");
  const [title, setTitle] = useState("");
  const [tags, setTags] = useState("");
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [currentUrl, currentTitle] = await Promise.all([
          getCurrentUrl(),
          getCurrentTitle(),
        ]);
        if (currentUrl) setUrl(currentUrl);
        if (currentTitle) setTitle(currentTitle);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const handleSave = async () => {
    if (!title.trim() || !url) return;
    setSaving(true);
    try {
      const tagList = tags.trim()
        ? tags.split(",").map((t) => t.trim()).filter(Boolean)
        : [];
      await saveReference({
        url,
        title: title.trim(),
        tags: tagList.length > 0 ? tagList : null,
      });
      getCurrentWindow().close();
    } finally {
      setSaving(false);
    }
  };

  const handleCancel = () => {
    getCurrentWindow().close();
  };

  return (
    <div className="save-ref-page">
      <h2 className="save-ref-page-title">Reference 저장</h2>

      {url && (
        <p className="save-ref-page-url">{url}</p>
      )}

      <input
        className="save-ref-page-input"
        placeholder="제목 입력"
        value={loading ? "" : title}
        onChange={(e) => setTitle(e.target.value)}
        autoFocus={!loading}
        disabled={loading}
      />

      <input
        className="save-ref-page-input"
        placeholder="태그 (쉼표로 구분, 선택)"
        value={tags}
        onChange={(e) => setTags(e.target.value)}
        disabled={loading}
      />

      <div className="save-ref-page-actions">
        <button
          className="save-ref-page-btn save-ref-page-btn--primary"
          onClick={handleSave}
          disabled={saving || !title.trim() || loading}
        >
          {saving ? "저장 중..." : "저장"}
        </button>
        <button
          className="save-ref-page-btn"
          onClick={handleCancel}
        >
          취소
        </button>
      </div>
    </div>
  );
}
