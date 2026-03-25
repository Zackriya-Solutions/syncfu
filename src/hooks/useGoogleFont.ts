import { useEffect, useState } from "react";

/** Track which fonts have already been injected to avoid duplicate <link> tags. */
const loadedFonts = new Set<string>();

/**
 * Dynamically loads a Google Font by name.
 * Returns true once the font is loaded and ready to render.
 *
 * Usage: `const ready = useGoogleFont("Space Grotesk");`
 * Then apply: `style={{ fontFamily: '"Space Grotesk", sans-serif' }}`
 */
export function useGoogleFont(fontName: string | undefined): boolean {
  const [ready, setReady] = useState(() =>
    fontName ? loadedFonts.has(fontName) : true
  );

  useEffect(() => {
    if (!fontName || loadedFonts.has(fontName)) {
      setReady(true);
      return;
    }

    const encoded = fontName.replace(/\s+/g, "+");
    const href = `https://fonts.googleapis.com/css2?family=${encoded}:wght@400;500;600;700&display=swap`;

    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = href;

    link.onload = () => {
      loadedFonts.add(fontName);
      setReady(true);
    };

    link.onerror = () => {
      // Font failed to load — fall back silently
      setReady(true);
    };

    document.head.appendChild(link);
  }, [fontName]);

  return ready;
}
