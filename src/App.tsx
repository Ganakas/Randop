import { useEffect, useState, type ReactNode } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const GITHUB = "github.com/Ganakas/Randop";

type Settings = {
  enabled: boolean;
  pick: boolean;
  open: boolean;
  multi: boolean;
  recursive: boolean;
};

type UpdateOffer = {
  available: boolean;
  version: string;
};

type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "ready"; version: string }
  | { kind: "downloading" }
  | { kind: "note"; text: string };

export function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState("");
  const [spin, setSpin] = useState(false);
  const [about, setAbout] = useState(false);
  const [version, setVersion] = useState("0.1.0");
  const [update, setUpdate] = useState<UpdateState>({ kind: "idle" });

  useEffect(() => {
    invoke<Settings>("settings")
      .then(setSettings)
      .catch((err: unknown) => setError(String(err)));
    getVersion()
      .then(setVersion)
      .catch(() => setVersion("0.1.0"));
  }, []);

  useEffect(() => {
    if (!about) return;
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") setAbout(false);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [about]);

  async function commit(next: Settings) {
    setSettings(next);
    setError("");
    try {
      const saved = await invoke<Settings>("save_settings", { state: next });
      setSettings(saved);
    } catch (err) {
      setError(String(err));
      const fresh = await invoke<Settings>("settings").catch(() => next);
      setSettings(fresh);
    }
  }

  async function checkUpdate() {
    setUpdate({ kind: "checking" });
    try {
      const offer = await invoke<UpdateOffer>("check_update");
      setUpdate(
        offer.available
          ? { kind: "ready", version: offer.version }
          : { kind: "current" },
      );
    } catch (err) {
      setUpdate({ kind: "note", text: String(err) });
    }
  }

  async function installUpdate() {
    setUpdate({ kind: "downloading" });
    try {
      await invoke("install_update");
    } catch (err) {
      setUpdate({ kind: "note", text: String(err) });
    }
  }

  function patch(part: Partial<Settings>) {
    if (!settings) return;
    void commit({ ...settings, ...part });
  }

  return (
    <div className="shell">
      <div className="card">
        <header
          className="header"
          onMouseDown={(event) => {
            if ((event.target as HTMLElement).closest("button")) return;
            void getCurrentWindow().startDragging();
          }}
        >
          <Die />
          <span className="title">Randop</span>
          <div className="window-buttons">
            <button
              className="glyph"
              aria-label="Свернуть"
              onClick={() => void getCurrentWindow().minimize()}
            >
              <Stroke>
                <path d="M5 12h14" />
              </Stroke>
            </button>
            <button
              className="glyph close"
              aria-label="Закрыть"
              onClick={() => void getCurrentWindow().close()}
            >
              <Stroke>
                <path d="M18 6 6 18" />
                <path d="m6 6 12 12" />
              </Stroke>
            </button>
          </div>
        </header>

        {settings && (
          <>
            <button
              className={settings.enabled ? "master on glass" : "master glass"}
              onClick={() => patch({ enabled: !settings.enabled })}
            >
              <span className="knob-track" aria-hidden="true">
                <span className="knob" />
              </span>
              <span>{settings.enabled ? "Включено" : "Выключено"}</span>
              <span className="led" aria-hidden="true" />
            </button>

            <p className="section">Пункты меню</p>
            <div className="group glass">
              <Check
                on={settings.pick}
                label="Случайно"
                onToggle={() => patch({ pick: !settings.pick })}
              />
              <Check
                on={settings.open}
                label="Открыть случайный"
                onToggle={() => patch({ open: !settings.open })}
              />
              <Check
                on={settings.multi}
                label="Выбрать несколько случайных"
                onToggle={() => patch({ multi: !settings.multi })}
              />
            </div>

            <p className="section">Поведение</p>
            <Check
              on={settings.recursive}
              label="Заглядывать в подпапки"
              onToggle={() => patch({ recursive: !settings.recursive })}
            />
          </>
        )}

        {error && <p className="error">{error}</p>}

        <div className="footer glass">
          <button className="icon-button" aria-label="Сведения" onClick={() => setAbout(true)}>
            <Stroke>
              <circle cx="12" cy="12" r="9" />
              <path d="M12 16v-4" />
              <path d="M12 8h.01" />
            </Stroke>
          </button>
          <button
            className={spin ? "icon-button spin" : "icon-button"}
            aria-label="Обновить меню"
            onClick={() => {
              if (!settings) return;
              setSpin(true);
              void commit(settings);
            }}
            onAnimationEnd={() => setSpin(false)}
          >
            <Stroke>
              <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
              <path d="M3 3v5h5" />
              <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
              <path d="M16 16h5v5" />
            </Stroke>
          </button>
        </div>

        {about && (
          <div className="about-layer" onMouseDown={() => setAbout(false)}>
            <div
              className="about glass"
              role="dialog"
              aria-label="О программе"
              onMouseDown={(event) => event.stopPropagation()}
            >
              <button className="about-close" aria-label="Закрыть" onClick={() => setAbout(false)}>
                <Stroke>
                  <path d="M18 6 6 18" />
                  <path d="m6 6 12 12" />
                </Stroke>
              </button>
              <Die className="about-die" />
              <p className="about-name">Randop</p>
              <p className="about-version">v{version}</p>
              <div className="update-row">
                <button
                  className="check-update"
                  disabled={update.kind === "checking" || update.kind === "downloading"}
                  onClick={() => void checkUpdate()}
                >
                  {update.kind === "checking"
                    ? "Проверка…"
                    : update.kind === "downloading"
                      ? "Скачивание…"
                      : "Проверить обновление"}
                </button>
                {update.kind === "ready" && (
                  <button
                    className="download"
                    aria-label={`Скачать версию ${update.version}`}
                    onClick={() => void installUpdate()}
                  >
                    <Stroke>
                      <path d="M12 4v11" />
                      <path d="m7 11 5 5 5-5" />
                      <path d="M5 20h14" />
                    </Stroke>
                  </button>
                )}
              </div>
              {update.kind === "current" && <p className="update-note">Это последняя версия</p>}
              {update.kind === "ready" && (
                <p className="update-note">Доступна версия {update.version}</p>
              )}
              {update.kind === "note" && <p className="update-note">{update.text}</p>}
              <div className="about-rule" />
              <button
                className="github"
                onClick={() => {
                  invoke("open_github").catch((err: unknown) => setError(String(err)));
                }}
              >
                <Stroke>
                  <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.4 5.4 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65S8.93 17.38 9 18v4" />
                  <path d="M9 18c-4.51 2-5-2-7-2" />
                </Stroke>
                <span>{GITHUB}</span>
                <Stroke>
                  <path d="M15 3h6v6" />
                  <path d="M10 14 21 3" />
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                </Stroke>
              </button>
              <svg className="heart" viewBox="0 0 24 24" aria-hidden="true">
                <path
                  fill="#e23d4a"
                  d="M12 20.5s-6.6-4.1-6.6-8.5a3.7 3.7 0 0 1 6.6-2.2 3.7 3.7 0 0 1 6.6 2.2c0 4.4-6.6 8.5-6.6 8.5z"
                />
              </svg>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function Check({
  on,
  label,
  onToggle,
}: {
  on: boolean;
  label: string;
  onToggle: () => void;
}) {
  return (
    <button className={on ? "row on" : "row"} onClick={onToggle}>
      <span className="box" aria-hidden="true">
        <svg className="tick" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M20 6 9 17l-5-5" />
        </svg>
      </span>
      <span>{label}</span>
    </button>
  );
}

function Die({ className = "die" }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 32 32" aria-hidden="true">
      <rect x="1.5" y="1.5" width="29" height="29" rx="8" fill="#EE7426" />
      <circle cx="11" cy="11" r="2.15" fill="#2A160C" />
      <circle cx="21" cy="11" r="2.15" fill="#2A160C" />
      <circle cx="16" cy="16" r="2.15" fill="#2A160C" />
      <circle cx="11" cy="21" r="2.15" fill="#2A160C" />
      <circle cx="21" cy="21" r="2.15" fill="#2A160C" />
    </svg>
  );
}

function Stroke({ children }: { children: ReactNode }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      {children}
    </svg>
  );
}
