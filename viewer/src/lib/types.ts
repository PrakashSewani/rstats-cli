export type CaptureScope = "standard" | "deep" | "unknown";

export interface ProcessRow {
  pid: number;
  name: string;
  user?: string;
  cpu: number;
  memBytes: number;
  command?: string;
}

export interface ProcessFrame {
  top: ProcessRow[];
}

export interface Session {
  source: "jsonl" | "export";
  scope: CaptureScope;
  fileName: string;
  hostname?: string;
  os?: string;
  startedAtMs: number;
  stoppedAtMs?: number;
  samples: number;
  timestamps: number[];
  cpu: number[];
  mem: number[];
  swap: number[];
  load: (number | null)[];
  netRx: number[];
  netTx: number[];
  disk: number[];
  perCore: number[][] | null;
  processes: ProcessFrame[] | null;
}
