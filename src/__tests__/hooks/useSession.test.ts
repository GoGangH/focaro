import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { useSession } from "../../hooks/useSession";

describe("useSession", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("초기 상태: 세션 없음", () => {
    const { result } = renderHook(() => useSession());
    expect(result.current.session).toBeNull();
    expect(result.current.isLoading).toBe(false);
  });

  it("startSession 호출 시 세션 상태 업데이트", async () => {
    mockIPC((cmd) => {
      if (cmd === "start_session") {
        return {
          id: "test-id",
          started_at: new Date().toISOString(),
          ended_at: null,
          status: "Active",
        };
      }
    });

    const { result } = renderHook(() => useSession());
    await act(async () => {
      await result.current.startSession();
    });
    expect(result.current.session).not.toBeNull();
    expect(result.current.session?.status).toBe("Active");
  });

  it("endSession 호출 시 세션 초기화", async () => {
    mockIPC((cmd) => {
      if (cmd === "start_session") {
        return { id: "test-id", started_at: new Date().toISOString(), ended_at: null, status: "Active" };
      }
      if (cmd === "end_session") return null;
    });

    const { result } = renderHook(() => useSession());
    await act(async () => { await result.current.startSession(); });
    await act(async () => { await result.current.endSession(); });
    expect(result.current.session).toBeNull();
  });
});
