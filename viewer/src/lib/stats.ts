import type { Session } from "./types";

export interface MetricSummary {
  avg: number | null;
  peak: number | null;
  peakOffsetMs: number | null;
}

export function summarize(values: (number | null)[], timestamps: number[]): MetricSummary {
  let sum = 0;
  let count = 0;
  let peak: number | null = null;
  let peakIndex = -1;
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (value == null || !Number.isFinite(value)) continue;
    sum += value;
    count += 1;
    if (peak === null || value > peak) {
      peak = value;
      peakIndex = index;
    }
  }
  const start = timestamps[0] ?? 0;
  return {
    avg: count > 0 ? sum / count : null,
    peak,
    peakOffsetMs: peakIndex >= 0 ? (timestamps[peakIndex] ?? start) - start : null,
  };
}

export function totalBytes(values: number[], timestamps: number[]): number {
  let total = 0;
  for (let index = 1; index < values.length; index += 1) {
    const dt = ((timestamps[index] ?? 0) - (timestamps[index - 1] ?? 0)) / 1000;
    if (dt > 0 && Number.isFinite(values[index])) total += values[index] * dt;
  }
  return total;
}

export interface SessionStats {
  cpu: MetricSummary;
  mem: MetricSummary;
  swap: MetricSummary;
  load: MetricSummary;
  disk: MetricSummary;
  receivedTotal: number;
  transmittedTotal: number;
}

export function buildStats(session: Session): SessionStats {
  return {
    cpu: summarize(session.cpu, session.timestamps),
    mem: summarize(session.mem, session.timestamps),
    swap: summarize(session.swap, session.timestamps),
    load: summarize(session.load, session.timestamps),
    disk: summarize(session.disk, session.timestamps),
    receivedTotal: totalBytes(session.netRx, session.timestamps),
    transmittedTotal: totalBytes(session.netTx, session.timestamps),
  };
}
