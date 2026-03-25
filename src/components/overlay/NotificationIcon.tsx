import { icons, type LucideProps } from "lucide-react";

interface NotificationIconProps extends LucideProps {
  readonly name: string;
}

/**
 * Renders a Lucide icon by name (e.g. "phone", "git-pull-request", "bell").
 *
 * Accepts kebab-case ("git-pull-request"), PascalCase ("GitPullRequest"),
 * or lowercase ("phone"). Returns null for unknown names.
 */
export function NotificationIcon({ name, ...props }: NotificationIconProps) {
  const Icon = resolveIcon(name);
  if (!Icon) return null;
  return <Icon {...props} />;
}

/** Convert any casing to PascalCase to match lucide-react's icon keys. */
function toPascalCase(str: string): string {
  return str
    .split(/[-_\s]+/)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join("");
}

function resolveIcon(name: string) {
  // Try exact match first (PascalCase)
  if (name in icons) {
    return icons[name as keyof typeof icons];
  }
  // Try converting to PascalCase from kebab/snake/lower
  const pascal = toPascalCase(name);
  if (pascal in icons) {
    return icons[pascal as keyof typeof icons];
  }
  return null;
}
