import { useEffect, useMemo, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "./lib/api";
import { appwriteEnabled, ensureSession, pushClip } from "./lib/appwrite";
import { kindMeta, timeAgo } from "./lib/format";
import type { Clip, ClipKind, TransformOp } from "./types";

const KIND_FILTERS: Array<ClipKind | "all"> = [
  "all",
  "text",
  "url",
  "code",
  "color",
  "image",
];

const TRANSFORMS: Array<{ op: TransformOp; label: string }> = [
  { op: "uppercase", label: "UPPER" },
  { op: "lowercase", label: "lower" },
  { op: "trim", label: "Trim" },
  { op: "url_encode", label: "URL" },
  { op: "json_pretty", label: "JSON" },
];

function App() {
  const win = getCurrentWindow();
  const isOverlay = win.label === "overlay";

  const [clips, setClips] = useState<Clip[]>([]);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<ClipKind | "all">("all");
  const [captureOn, setCaptureOn] = useState(true);
  const [workspace, setWorkspace] = useState("default");
  const [toast, setToast] = useState<string | null>(null);

  const flash = useCallback((msg: string) => {
    setToast(msg);
    window.setTimeout(() => setToast(null), 1600);
  }, []);

  // Initial load + live subscription.
  useEffect(() => {
    api.getClips().then(setClips).catch(console.error);
    api.getCaptureEnabled().then(setCaptureOn).catch(console.error);
    api.getWorkspace().then(setWorkspace).catch(console.error);
    if (appwriteEnabled) ensureSession();

    const unlisten = listen<Clip>("clip-added", (event) => {
      const clip = event.payload;
      setClips((prev) => [clip, ...prev.filter((c) => c.id !== clip.id)]);
      if (appwriteEnabled) pushClip(clip);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Overlay: hide on Escape.
  useEffect(() => {
    if (!isOverlay) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") win.hide();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isOverlay, win]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return clips.filter((c) => {
      if (filter !== "all" && c.kind !== filter) return false;
      if (!q) return true;
      return (
        c.preview.toLowerCase().includes(q) ||
        c.content.toLowerCase().includes(q) ||
        c.tags.some((t) => t.toLowerCase().includes(q)) ||
        c.kind.includes(q)
      );
    });
  }, [clips, query, filter]);

  const copyClip = useCallback(
    async (clip: Clip) => {
      if (clip.kind === "image") {
        flash("Image copy lands in a later phase");
        return;
      }
      await api.writeClipboard(clip.content);
      flash("Copied to clipboard");
      if (isOverlay) win.hide();
    },
    [flash, isOverlay, win],
  );

  const transformAndCopy = useCallback(
    async (clip: Clip, op: TransformOp) => {
      try {
        const out = await api.transformText(clip.content, op);
        await api.writeClipboard(out);
        flash(`Pasted as ${op}`);
      } catch (e) {
        flash(String(e));
      }
    },
    [flash],
  );

  const removeClip = useCallback(async (id: string) => {
    await api.deleteClip(id);
    setClips((prev) => prev.filter((c) => c.id !== id));
  }, []);

  const toggleCapture = useCallback(async () => {
    const next = !captureOn;
    await api.setCaptureEnabled(next);
    setCaptureOn(next);
    flash(next ? "Capture resumed" : "Incognito: capture paused");
  }, [captureOn, flash]);

  const clearAll = useCallback(async () => {
    await api.clearClips();
    setClips([]);
  }, []);

  const commitWorkspace = useCallback(
    async (name: string) => {
      const clean = name.trim() || "default";
      setWorkspace(clean);
      await api.setWorkspace(clean);
    },
    [],
  );

  return (
    <div className={`app ${isOverlay ? "overlay" : ""}`}>
      <header className="topbar">
        <div className="brand">
          <span className="logo">◳</span>
          <div>
            <h1>{isOverlay ? "Power Paste" : "Smart Clipboard"}</h1>
            <p className="subtitle">
              {isOverlay
                ? "Pick a clip · Esc to close"
                : "Your clipboard, as a second brain"}
            </p>
          </div>
        </div>
        {!isOverlay && (
          <div className="topbar-actions">
            <label className="ws">
              <span>Workspace</span>
              <input
                value={workspace}
                onChange={(e) => setWorkspace(e.target.value)}
                onBlur={(e) => commitWorkspace(e.target.value)}
              />
            </label>
            <button
              className={`pill ${captureOn ? "on" : "off"}`}
              onClick={toggleCapture}
              title="Toggle capture (incognito)"
            >
              {captureOn ? "● Capturing" : "⏸ Paused"}
            </button>
            <button className="ghost" onClick={clearAll}>
              Clear all
            </button>
          </div>
        )}
      </header>

      <div className="controls">
        <input
          className="search"
          autoFocus={isOverlay}
          placeholder="Search clips…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <div className="filters">
          {KIND_FILTERS.map((k) => (
            <button
              key={k}
              className={`chip ${filter === k ? "active" : ""}`}
              onClick={() => setFilter(k)}
            >
              {k}
            </button>
          ))}
        </div>
      </div>

      {!appwriteEnabled && !isOverlay && (
        <div className="banner">
          Appwrite sync not configured — running locally. Add your endpoint &
          project in <code>.env</code> to enable cloud sync.
        </div>
      )}

      <main className="list">
        {filtered.length === 0 ? (
          <div className="empty">
            <p>No clips yet.</p>
            <p className="hint">Copy something — it shows up here instantly.</p>
          </div>
        ) : (
          filtered.map((clip) => (
            <ClipCard
              key={clip.id}
              clip={clip}
              onCopy={copyClip}
              onDelete={removeClip}
              onTransform={transformAndCopy}
              compact={isOverlay}
            />
          ))
        )}
      </main>

      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}

function ClipCard({
  clip,
  onCopy,
  onDelete,
  onTransform,
  compact,
}: {
  clip: Clip;
  onCopy: (c: Clip) => void;
  onDelete: (id: string) => void;
  onTransform: (c: Clip, op: TransformOp) => void;
  compact: boolean;
}) {
  const [revealed, setRevealed] = useState(false);
  const meta = kindMeta[clip.kind];
  const masked = clip.sensitive && !revealed;

  return (
    <article className="card" onDoubleClick={() => onCopy(clip)}>
      <div className="card-head">
        <span className="badge" style={{ background: meta.color }}>
          {meta.label}
        </span>
        {clip.sensitive && <span className="badge sensitive">SENSITIVE</span>}
        {clip.kind === "color" && (
          <span
            className="swatch"
            style={{ background: String(clip.meta.hex ?? clip.content) }}
          />
        )}
        <span className="time">{timeAgo(clip.created_at)}</span>
      </div>

      <div className="card-body">
        {clip.kind === "image" ? (
          <img className="thumb" src={clip.content} alt={clip.preview} />
        ) : masked ? (
          <button className="reveal" onClick={() => setRevealed(true)}>
            •••••• click to reveal
          </button>
        ) : (
          <pre className={`content ${clip.kind === "code" ? "code" : ""}`}>
            {clip.preview}
          </pre>
        )}
      </div>

      <div className="card-actions">
        <button onClick={() => onCopy(clip)}>Copy</button>
        {!compact && clip.kind !== "image" && (
          <div className="transforms">
            {TRANSFORMS.map((t) => (
              <button
                key={t.op}
                className="tiny"
                title={`Paste as ${t.op}`}
                onClick={() => onTransform(clip, t.op)}
              >
                {t.label}
              </button>
            ))}
          </div>
        )}
        <button className="danger" onClick={() => onDelete(clip.id)}>
          Delete
        </button>
      </div>
    </article>
  );
}

export default App;
