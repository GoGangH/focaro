import { useState } from "react";
import type { Reference } from "../../types/bindings";
import { saveReference } from "../../services/reference";

interface SaveReferenceProps {
  currentUrl: string | null;
  onSaved?: (ref: Reference) => void;
}

export function SaveReference({ currentUrl, onSaved }: SaveReferenceProps) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [tags, setTags] = useState("");
  const [saving, setSaving] = useState(false);

  const handleOpen = () => {
    if (!currentUrl) return;
    setTitle("");
    setTags("");
    setOpen(true);
  };

  const handleClose = () => {
    setOpen(false);
  };

  const handleSave = async () => {
    if (!title.trim()) return;
    setSaving(true);
    try {
      const tagList = tags.trim()
        ? tags.split(",").map((t) => t.trim()).filter(Boolean)
        : [];
      const ref = await saveReference({
        url: currentUrl!,
        title: title.trim(),
        tags: tagList.length > 0 ? tagList : null,
      });
      onSaved?.(ref);
      setOpen(false);
    } finally {
      setSaving(false);
    }
  };

  if (open) {
    return (
      <div className="save-ref-form">
        <input
          className="save-ref-form__input"
          value={currentUrl ?? ""}
          readOnly
          style={{ color: "#636366", fontSize: 11 }}
        />
        <input
          className="save-ref-form__input"
          placeholder="제목 입력"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          autoFocus
        />
        <input
          className="save-ref-form__input"
          placeholder="태그 (쉼표로 구분, 선택)"
          value={tags}
          onChange={(e) => setTags(e.target.value)}
        />
        <div className="save-ref-form__actions">
          <button
            className="session-btn session-btn--start"
            style={{ flex: 1, padding: "7px 0", fontSize: 13 }}
            onClick={handleSave}
            disabled={saving || !title.trim()}
          >
            저장
          </button>
          <button
            className="dashboard-btn"
            style={{ flex: 1 }}
            onClick={handleClose}
          >
            취소
          </button>
        </div>
      </div>
    );
  }

  return (
    <button
      className={`dashboard-btn${currentUrl ? " dashboard-btn--active" : ""}`}
      onClick={handleOpen}
      disabled={!currentUrl}
    >
      Reference 저장
    </button>
  );
}
