import { useCallback, useRef, useState } from "react";

export function useToast() {
  const [message, setMessage] = useState<string | null>(null);
  const [leaving, setLeaving] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const clear = useCallback(() => {
    if (timer.current) clearTimeout(timer.current);
    setLeaving(true);
    timer.current = setTimeout(() => {
      setMessage(null);
      setLeaving(false);
    }, 260);
  }, []);

  const show = useCallback(
    (text: string, ms = 4000) => {
      if (timer.current) clearTimeout(timer.current);
      setLeaving(false);
      setMessage(text);
      if (ms > 0) {
        timer.current = setTimeout(() => clear(), ms);
      }
    },
    [clear],
  );

  return { message, leaving, show, clear };
}
