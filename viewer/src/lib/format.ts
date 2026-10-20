const KIB = 1024;

export function fmtBytes(bytes: number, perSecond = false): string {
  const suffix = perSecond ? "/s" : "";
  if (!Number.isFinite(bytes)) return "–";
  const abs = Math.abs(bytes);
  if (abs < KIB) return `${Math.round(bytes)} B${suffix}`;
  const units = ["KiB", "MiB", "GiB", "TiB"];
  let value = bytes / KIB;
  let unit = 0;
  while (Math.abs(value) >= KIB && unit < units.length - 1) {
    value /= KIB;
    unit += 1;
  }
  const digits = Math.abs(value) >= 100 ? 0 : Math.abs(value) >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[unit]}${suffix}`;
}

export function fmtPct(value: number | null | undefined, digits = 1): string {
  return value == null || !Number.isFinite(value) ? "–" : `${value.toFixed(digits)}%`;
}

export function fmtLoad(value: number | null | undefined): string {
  return value == null || !Number.isFinite(value) ? "–" : value.toFixed(2);
}

export function fmtDuration(ms: number): string {
  if (!Number.isFinite(ms) || ms < 0) return "–";
  const total = Math.round(ms / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  if (hours > 0) {
    return `${hours}h ${String(minutes).padStart(2, "0")}m ${String(seconds).padStart(2, "0")}s`;
  }
  if (minutes > 0) return `${minutes}m ${String(seconds).padStart(2, "0")}s`;
  return `${seconds}s`;
}

export function fmtOffset(ms: number): string {
  const total = Math.max(0, Math.round(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  if (hours > 0) {
    return `+${hours}h ${String(minutes).padStart(2, "0")}m`;
  }
  if (minutes > 0) return `+${minutes}m ${String(seconds).padStart(2, "0")}s`;
  return `+${seconds}s`;
}

export function fmtClock(ms: number): string {
  const date = new Date(ms);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

export function fmtDateTime(ms: number): string {
  const date = new Date(ms);
  const day = date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
  return `${day} · ${fmtClock(ms)}`;
}

export function fmtElapsed(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  const pad = (value: number) => String(value).padStart(2, "0");
  if (hours > 0) return `${hours}:${pad(minutes)}:${pad(secs)}`;
  return `${minutes}:${pad(secs)}`;
}
