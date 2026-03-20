import { useState, useCallback } from "react";
import type { Session } from "../types/bindings";
import * as sessionService from "../services/session";

interface UseSessionReturn {
  session: Session | null;
  isLoading: boolean;
  startSession: () => Promise<void>;
  endSession: () => Promise<void>;
}

export function useSession(): UseSessionReturn {
  const [session, setSession] = useState<Session | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const startSession = useCallback(async () => {
    setIsLoading(true);
    try {
      const newSession = await sessionService.startSession();
      setSession(newSession);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const endSession = useCallback(async () => {
    setIsLoading(true);
    try {
      await sessionService.endSession();
      setSession(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { session, isLoading, startSession, endSession };
}
