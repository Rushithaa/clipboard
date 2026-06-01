import type { ClipKind } from "../types";

export function timeAgo(iso: string): string {
  const then = new Date(iso).getTime();
  const secs = Math.max(0, Math.floor((Date.now() - then) / 1000));
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export const kindMeta: Record<ClipKind, { label: string; color: string }> = {
  text: { label: "TEXT", color: "#64748b" },
  url: { label: "URL", color: "#2563eb" },
  code: { label: "CODE", color: "#7c3aed" },
  color: { label: "COLOR", color: "#0d9488" },
  image: { label: "IMAGE", color: "#db2777" },
};
