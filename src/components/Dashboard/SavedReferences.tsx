import { invoke } from "@tauri-apps/api/core";
import type { Reference } from "../../types/bindings";

interface Props {
  references: Reference[];
}

export function SavedReferences({ references }: Props) {
  if (references.length === 0) {
    return <p className="dash-empty">저장된 Reference 없음</p>;
  }

  const handleOpen = async (url: string) => {
    try {
      await invoke("open_url", { url });
    } catch {
      // 실패 시 무시
    }
  };

  return (
    <div className="saved-refs">
      {references.map((ref) => (
        <div key={ref.id} className="saved-ref-item">
          <button
            className="saved-ref-title"
            onClick={() => handleOpen(ref.url)}
          >
            {ref.title}
          </button>
          <div className="saved-ref-url">{ref.url}</div>
          {ref.tags.length > 0 && (
            <div className="saved-ref-tags">
              {ref.tags.map((tag) => (
                <span key={tag} className="saved-ref-tag">{tag}</span>
              ))}
            </div>
          )}
          <div className="saved-ref-date">
            {new Date(ref.created_at).toLocaleDateString("ko-KR")}
          </div>
        </div>
      ))}
    </div>
  );
}
