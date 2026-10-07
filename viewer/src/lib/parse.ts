import type { ProcessFrame, ProcessRow, Session } from "./types";

const PROCESS_TOP_CPU = 24;
const PROCESS_TOP_MEM = 12;
const COMMAND_LIMIT = 140;
const SNIFF_BYTES = 4096;
const INLINE_PARSE_LIMIT = 5 * 1024 * 1024;

export class ParseError extends Error {}

type JsonObject = Record<string, unknown>;

function asObject(value: unknown): JsonObject | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as JsonObject)
    : null;
}

function asArray(value: unknown): unknown[] | null {
  return Array.isArray(value) ? value : null;
}

function num(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function str(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function tryParseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

class SessionBuilder {
  source: Session["source"];
  scope: Session["scope"] = "unknown";
  fileName: string;
  hostname?: string;
  os?: string;
  startedAtMs?: number;
  stoppedAtMs?: number;
  timestamps: number[] = [];
  cpu: number[] = [];
  mem: number[] = [];
  swap: number[] = [];
  load: (number | null)[] = [];
  netRx: number[] = [];
  netTx: number[] = [];
  disk: number[] = [];
  perCore: number[][] | null = null;

  private processes: ProcessFrame[] | null = null;
  private hasProcessData = false;
  private prevNet: { ts: number; interfaces: Map<string, { rx: number; tx: number }> } | null = null;

  constructor(source: Session["source"], fileName: string) {
    this.source = source;
    this.fileName = fileName;
  }

  accept(record: JsonObject): void {
    const type = str(record.type);
    if (type === "header") {
      const scope = str(record.scope);
      if (scope === "standard" || scope === "deep") this.scope = scope;
      const started = num(record.started_at_ms);
      if (started !== null && this.startedAtMs === undefined) this.startedAtMs = started;
      return;
    }
    if (type === "sample") {
      const snapshot = asObject(record.snapshot);
      if (snapshot) this.addSnapshot(snapshot);
      return;
    }
    if (type === "footer") {
      const stopped = num(record.stopped_at_ms);
      if (stopped !== null) this.stoppedAtMs = stopped;
    }
  }

  addExportRow(row: JsonObject): void {
    const timestamp = num(row.timestamp_ms);
    if (timestamp === null) return;
    this.pushSample(timestamp);
    this.cpu.push(num(row.cpu_total_pct) ?? 0);
    this.mem.push(num(row.memory_used_pct) ?? 0);
    this.swap.push(num(row.swap_used_pct) ?? 0);
    this.load.push(num(row.load_average));
    this.netRx.push(num(row.net_received_bps) ?? 0);
    this.netTx.push(num(row.net_transmitted_bps) ?? 0);
    this.disk.push(num(row.disk_max_used_pct) ?? 0);
  }

  addSnapshot(snapshot: JsonObject): void {
    const timestamp = num(snapshot.timestamp);
    if (timestamp === null) return;
    this.pushSample(timestamp);

    const cpu = asObject(snapshot.cpu);
    const cpuTotal = cpu ? num(cpu.total_usage) : null;
    this.cpu.push(cpuTotal ?? 0);

    const memory = asObject(snapshot.memory);
    this.mem.push(memory ? num(memory.used_percent) ?? 0 : 0);
    const swap = asObject(snapshot.swap);
    this.swap.push(swap ? num(swap.used_percent) ?? 0 : 0);
    this.load.push(num(snapshot.load_average));

    let diskMax = 0;
    const disks = asArray(snapshot.disks);
    if (disks) {
      for (const disk of disks) {
        const entry = asObject(disk);
        const used = entry ? num(entry.used_percent) : null;
        if (used !== null && used > diskMax) diskMax = used;
      }
    }
    this.disk.push(diskMax);

    this.ingestNetwork(timestamp, snapshot);
    this.ingestPerCore(cpu);

    if (this.hostname === undefined) this.hostname = str(snapshot.hostname);
    if (this.os === undefined) this.os = str(snapshot.os);

    this.ingestProcesses(snapshot);
  }

  private pushSample(timestamp: number): void {
    this.timestamps.push(timestamp);
  }

  private ingestNetwork(timestamp: number, snapshot: JsonObject): void {
    const interfaces = new Map<string, { rx: number; tx: number }>();
    const networks = asArray(snapshot.networks);
    if (networks) {
      for (const network of networks) {
        const entry = asObject(network);
        const name = entry ? str(entry.name) : undefined;
        if (!entry || name === undefined) continue;
        interfaces.set(name, {
          rx: num(entry.received_bytes) ?? 0,
          tx: num(entry.transmitted_bytes) ?? 0,
        });
      }
    }
    let rxRate = 0;
    let txRate = 0;
    if (this.prevNet && timestamp > this.prevNet.ts) {
      const dt = (timestamp - this.prevNet.ts) / 1000;
      for (const [name, current] of interfaces) {
        const previous = this.prevNet.interfaces.get(name);
        if (!previous) continue;
        const deltaRx = current.rx - previous.rx;
        const deltaTx = current.tx - previous.tx;
        if (deltaRx > 0) rxRate += deltaRx / dt;
        if (deltaTx > 0) txRate += deltaTx / dt;
      }
    }
    this.netRx.push(rxRate);
    this.netTx.push(txRate);
    this.prevNet = { ts: timestamp, interfaces };
  }

  private ingestPerCore(cpu: JsonObject | null): void {
    const cores = cpu ? asArray(cpu.per_core) : null;
    if (cores) {
      const values = cores.map((core) => num(core) ?? Number.NaN);
      if (!this.perCore) this.perCore = values.map(() => []);
      const width = Math.min(this.perCore.length, values.length);
      for (let index = 0; index < width; index += 1) {
        this.perCore[index].push(values[index]);
      }
    } else if (this.perCore) {
      for (const series of this.perCore) series.push(Number.NaN);
    }
  }

  private ingestProcesses(snapshot: JsonObject): void {
    const processes = asArray(snapshot.processes);
    if (processes && processes.length > 0) {
      this.hasProcessData = true;
      if (!this.processes) {
        this.processes = Array.from({ length: this.timestamps.length - 1 }, () => ({ top: [] }));
      }
      this.processes.push({ top: topProcesses(processes) });
      return;
    }
    if (this.processes) this.processes.push({ top: [] });
  }

  finish(): Session {
    if (this.timestamps.length === 0) {
      throw new ParseError(
        "No samples found in this file — expected an rstats recording (.jsonl) or an export file.",
      );
    }
    const samples = this.timestamps.length;
    let scope = this.scope;
    if (scope === "unknown") scope = this.hasProcessData ? "deep" : "standard";
    if (this.source === "export") scope = "unknown";

    let processes = this.processes;
    if (this.hasProcessData && processes) {
      while (processes.length < samples) processes.push({ top: [] });
      processes = processes.slice(0, samples);
    } else {
      processes = null;
    }

    const startedAtMs = this.startedAtMs ?? this.timestamps[0];
    return {
      source: this.source,
      scope,
      fileName: this.fileName,
      hostname: this.hostname,
      os: this.os,
      startedAtMs,
      stoppedAtMs: this.stoppedAtMs,
      samples,
      timestamps: this.timestamps,
      cpu: this.cpu,
      mem: this.mem,
      swap: this.swap,
      load: this.load,
      netRx: this.netRx,
      netTx: this.netTx,
      disk: this.disk,
      perCore: this.perCore,
      processes,
    };
  }
}

function processKey(row: ProcessRow): string {
  return row.pid !== 0 ? `pid:${row.pid}` : `name:${row.name}`;
}

function topProcesses(raw: unknown[]): ProcessRow[] {
  const rows: ProcessRow[] = [];
  for (const value of raw) {
    const entry = asObject(value);
    if (!entry) continue;
    rows.push({
      pid: num(entry.pid) ?? 0,
      name: str(entry.name) ?? "unknown",
      user: str(entry.user),
      cpu: num(entry.cpu_usage) ?? 0,
      memBytes: num(entry.memory_bytes) ?? 0,
      command: str(entry.command),
    });
  }
  const byCpu = [...rows].sort((left, right) => right.cpu - left.cpu).slice(0, PROCESS_TOP_CPU);
  const byMem = [...rows].sort((left, right) => right.memBytes - left.memBytes).slice(0, PROCESS_TOP_MEM);
  const seen = new Set<string>();
  const top: ProcessRow[] = [];
  for (const row of [...byCpu, ...byMem]) {
    const key = processKey(row);
    if (seen.has(key)) continue;
    seen.add(key);
    if (row.command && row.command.length > COMMAND_LIMIT) {
      row.command = `${row.command.slice(0, COMMAND_LIMIT - 1)}…`;
    }
    top.push(row);
  }
  return top;
}

function parseExportArray(text: string, fileName: string): Session {
  const value = tryParseJson(text);
  if (value === undefined) {
    throw new ParseError("This file starts like a JSON array but could not be parsed.");
  }
  const rows = asArray(value);
  if (!rows) throw new ParseError("This file is not a JSON array.");
  const builder = new SessionBuilder("export", fileName);
  for (const row of rows) {
    const entry = asObject(row);
    if (entry) builder.addExportRow(entry);
  }
  return builder.finish();
}

function parseSingleJson(value: unknown, fileName: string): Session {
  const record = asObject(value);
  if (!record) throw new ParseError("Unrecognized file contents.");
  const builder = new SessionBuilder("jsonl", fileName);
  if (str(record.type) !== undefined) {
    builder.accept(record);
    return builder.finish();
  }
  if (num(record.timestamp_ms) !== null) {
    builder.source = "export";
    builder.addExportRow(record);
    return builder.finish();
  }
  if (record.cpu !== undefined && num(record.timestamp) !== null) {
    builder.addSnapshot(record);
    return builder.finish();
  }
  throw new ParseError(
    "Unrecognized file — expected an rstats recording (.jsonl) or an export file (--export --format json).",
  );
}

function parseJsonlLines(lines: string[], fileName: string): Session {
  const builder = new SessionBuilder("jsonl", fileName);
  for (const line of lines) {
    const parsed = tryParseJson(line);
    const record = parsed === undefined ? null : asObject(parsed);
    if (record) builder.accept(record);
  }
  return builder.finish();
}

export function parseText(text: string, fileName: string): Session {
  const trimmed = text.replace(/^\uFEFF/, "").trimStart();
  if (trimmed.length === 0) throw new ParseError("This file is empty.");
  if (trimmed.startsWith("[")) return parseExportArray(trimmed, fileName);
  if (trimmed.startsWith("{")) {
    const whole = tryParseJson(text);
    if (whole !== undefined) return parseSingleJson(whole, fileName);
  }
  const lines = text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  return parseJsonlLines(lines, fileName);
}

async function* readLines(file: File): AsyncGenerator<string> {
  const stream = file.stream().pipeThrough(new TextDecoderStream());
  const reader = stream.getReader();
  let buffer = "";
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    buffer += value;
    let index = buffer.indexOf("\n");
    while (index >= 0) {
      yield buffer.slice(0, index);
      buffer = buffer.slice(index + 1);
      index = buffer.indexOf("\n");
    }
  }
  if (buffer.trim().length > 0) yield buffer;
}

export async function parseFile(file: File): Promise<Session> {
  const head = (await file.slice(0, SNIFF_BYTES).text()).replace(/^\uFEFF/, "").trimStart();
  if (!head.startsWith("[") && file.size > INLINE_PARSE_LIMIT) {
    const builder = new SessionBuilder("jsonl", file.name);
    for await (const line of readLines(file)) {
      const trimmed = line.trim();
      if (trimmed.length === 0) continue;
      const parsed = tryParseJson(trimmed);
      const record = parsed === undefined ? null : asObject(parsed);
      if (record) builder.accept(record);
    }
    return builder.finish();
  }
  return parseText(await file.text(), file.name);
}
