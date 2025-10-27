import { useEffect, useState } from "react";

export function usePlaceholderRotator(placeholders: string[], intervalMs = 3000) {
  const [currentIndex, setCurrentIndex] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentIndex((prev) => (prev + 1) % placeholders.length);
    }, intervalMs);

    return () => clearInterval(timer);
  }, [placeholders.length, intervalMs]);

  return placeholders[currentIndex];
}
