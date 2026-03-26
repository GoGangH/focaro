import { useState, useMemo } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Reference } from "../../types/bindings";
import { deleteReference, updateReference } from "../../services/reference";

interface Props {
  references: Reference[];
  onRefresh: () => void;
}

function faviconUrl(url: string): string {
  try {
    const hostname = new URL(url).hostname.replace(/^www\./, "");
    return `https://www.google.com/s2/favicons?domain=${hostname}&sz=32`;
  } catch {
    return "";
  }
}

interface EditState {
  id: string;
  title: string;
  tags: string;
}

export function SavedReferences({ references, onRefresh }: Props) {
  const [editing, setEditing] = useState<EditState | null>(null);
  const [saving, setSaving] = useState(false);
  const [search, setSearch] = useState("");
  const [activeTag, setActiveTag] = useState<string | null>(null);

  const allTags = useMemo(() => {
    const set = new Set<string>();
    for (const ref of references) {
      for (const tag of ref.tags) set.add(tag);
    }
    return Array.from(set).sort();
  }, [references]);

  const filtered = useMemo(() => {
    return references.filter((ref) => {
      const matchesSearch =
        !search ||
        ref.title.toLowerCase().includes(search.toLowerCase()) ||
        ref.url.toLowerCase().includes(search.toLowerCase());
      const matchesTag = !activeTag || ref.tags.includes(activeTag);
      return matchesSearch && matchesTag;
    });
  }, [references, search, activeTag]);

  const handleOpen = async (url: string) => {
    try {
      await openUrl(url);
    } catch {
      // 실패 시 무시
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteReference(id);
      onRefresh();
    } catch {
      // 실패 시 무시
    }
  };

  const handleEditStart = (ref: Reference) => {
    setEditing({ id: ref.id, title: ref.title, tags: ref.tags.join(", ") });
  };

  const handleEditSave = async () => {
    if (!editing || !editing.title.trim()) return;
    setSaving(true);
    try {
      const tagList = editing.tags.trim()
        ? editing.tags.split(",").map((t) => t.trim()).filter(Boolean)
        : [];
      await updateReference({ id: editing.id, title: editing.title.trim(), tags: tagList });
      setEditing(null);
      onRefresh();
    } finally {
      setSaving(false);
    }
  };

  if (references.length === 0) {
    return <p className="dash-empty">저장된 Reference 없음</p>;
  }

  return (
    <div className="saved-refs">
      <div className="saved-refs__filters">
        <input
          className="saved-refs__search"
          type="text"
          placeholder="제목 또는 URL 검색..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        {allTags.length > 0 && (
          <div className="saved-refs__tags">
            <button
              className={`saved-ref-tag saved-ref-tag--filter${activeTag === null ? " active" : ""}`}
              onClick={() => setActiveTag(null)}
            >
              전체
            </button>
            {allTags.map((tag) => (
              <button
                key={tag}
                className={`saved-ref-tag saved-ref-tag--filter${activeTag === tag ? " active" : ""}`}
                onClick={() => setActiveTag(activeTag === tag ? null : tag)}
              >
                {tag}
              </button>
            ))}
          </div>
        )}
      </div>

      {filtered.length === 0 && (
        <p className="dash-empty">검색 결과 없음</p>
      )}

      {filtered.map((ref) => {
        const isEditing = editing?.id === ref.id;
        return (
          <div key={ref.id} className="saved-ref-item">
            {isEditing ? (
              <div className="saved-ref-edit">
                <input
                  className="save-ref-form__input"
                  value={editing.title}
                  onChange={(e) => setEditing({ ...editing, title: e.target.value })}
                  autoFocus
                />
                <input
                  className="save-ref-form__input"
                  placeholder="태그 (쉼표로 구분)"
                  value={editing.tags}
                  onChange={(e) => setEditing({ ...editing, tags: e.target.value })}
                />
                <div className="saved-ref-actions">
                  <button
                    className="session-btn session-btn--start"
                    style={{ fontSize: 12, padding: "4px 10px" }}
                    onClick={handleEditSave}
                    disabled={saving || !editing.title.trim()}
                  >
                    저장
                  </button>
                  <button
                    className="dashboard-btn"
                    style={{ fontSize: 12, padding: "4px 10px" }}
                    onClick={() => setEditing(null)}
                  >
                    취소
                  </button>
                </div>
              </div>
            ) : (
              <>
                <div className="saved-ref-header">
                  <img
                    className="saved-ref-favicon"
                    src={faviconUrl(ref.url)}
                    alt=""
                    width={16}
                    height={16}
                    onError={(e) => {
                      (e.currentTarget as HTMLImageElement).style.display = "none";
                    }}
                  />
                  <button
                    className="saved-ref-title"
                    onClick={() => handleOpen(ref.url)}
                  >
                    {ref.title}
                  </button>
                  <div className="saved-ref-controls">
                    <button
                      className="saved-ref-icon-btn"
                      onClick={() => handleEditStart(ref)}
                      title="수정"
                    >
                      ✏️
                    </button>
                    <button
                      className="saved-ref-icon-btn"
                      onClick={() => handleDelete(ref.id)}
                      title="삭제"
                    >
                      🗑️
                    </button>
                  </div>
                </div>
                {ref.tags.length > 0 && (
                  <div className="saved-ref-tags">
                    {ref.tags.map((tag) => (
                      <span key={tag} className="saved-ref-tag">
                        {tag}
                      </span>
                    ))}
                  </div>
                )}
                <div className="saved-ref-date">
                  {new Date(ref.created_at).toLocaleDateString("ko-KR")}
                </div>
              </>
            )}
          </div>
        );
      })}
    </div>
  );
}
