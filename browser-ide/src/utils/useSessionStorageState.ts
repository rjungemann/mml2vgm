import { useCallback, useEffect, useState } from 'react';

type SetStateAction<T> = T | ((prevState: T) => T);

const isBrowser = typeof window !== 'undefined';

export function useSessionStorageState<T>(
  key: string,
  initialValue: T
): [T, (value: SetStateAction<T>) => void] {
  const [state, setState] = useState<T>(() => {
    if (!isBrowser) {
      return initialValue;
    }

    try {
      const raw = window.sessionStorage.getItem(key);
      if (raw == null) {
        return initialValue;
      }
      return JSON.parse(raw) as T;
    } catch {
      return initialValue;
    }
  });

  useEffect(() => {
    if (!isBrowser) {
      return;
    }

    try {
      window.sessionStorage.setItem(key, JSON.stringify(state));
    } catch {
      // Ignore quota/unavailable storage errors and keep state in-memory.
    }
  }, [key, state]);

  const setSessionState = useCallback((value: SetStateAction<T>) => {
    setState((prev) => (typeof value === 'function' ? (value as (prevState: T) => T)(prev) : value));
  }, []);

  return [state, setSessionState];
}