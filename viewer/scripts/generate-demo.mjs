import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outDir = path.join(root, "public", "demo");

const START_MS = Date.UTC(2026, 9, 7, 9, 12, 0);
const CORES = 8;
const MiB = 1024 * 1024;

function mulberry32(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let t = state;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const clamp = (value, lo, hi) => Math.min(hi, Math.max(lo, value));
const round = (value, digits = 1) => Number(value.toFixed(digits));
const walk = (previous, rng, step, lo, hi) => clamp(previous + (rng() - 0.5) * step, lo, hi);

function header(scope) {
  return JSON.stringify({ type: "header", version: 1, scope, started_at_ms: START_MS });
}

function footer(count, intervalMs) {
  return JSON.stringify({
    type: "footer",
    stopped_at_ms: START_MS + (count - 1) * intervalMs,
    samples: count,
  });
}

function generateStandard() {
  const rng = mulberry32(41);
  const intervalMs = 10_000;
  const count = 121;
  const lines = [header("standard")];
  let mem = 54.0;
  let swap = 2.5;
  let diskA = 62.4;
  let diskB = 70.8;
  const nets = {
    en0: { rx: 512 * MiB, tx: 128 * MiB },
    utun0: { rx: 12 * MiB, tx: 4 * MiB },
  };
  for (let index = 0; index < count; index += 1) {
    const timestamp = START_MS + index * intervalMs;
    const seconds = (index * intervalMs) / 1000;
    const burst = index >= 36 && index <= 44 ? 38 : 0;
    const cpu = clamp(17 + 9 * Math.sin(seconds / 260) + (rng() - 0.5) * 7 + burst, 2, 97);
    mem = walk(mem, rng, 1.2, 38, 78);
    swap = walk(swap, rng, 0.5, 0, 9);
    diskA = walk(diskA, rng, 0.15, 55, 80);
    diskB = walk(diskB, rng, 0.2, 60, 88);
    const rxRate = clamp(180_000 + 390_000 * (0.5 + 0.5 * Math.sin(seconds / 90)) * rng() + burst * 42_000, 20_000, 4_200_000);
    const txRate = clamp(55_000 * (0.6 + rng()) + burst * 9_000, 8_000, 900_000);
    nets.en0.rx += rxRate * 0.94 * (intervalMs / 1000);
    nets.en0.tx += txRate * 0.96 * (intervalMs / 1000);
    nets.utun0.rx += rxRate * 0.05 * (intervalMs / 1000);
    nets.utun0.tx += txRate * 0.03 * (intervalMs / 1000);
    const perCore = [];
    for (let core = 0; core < CORES; core += 1) {
      perCore.push(round(clamp(cpu + (rng() - 0.5) * 26, 0, 100), 1));
    }
    lines.push(
      JSON.stringify({
        type: "sample",
        snapshot: {
          timestamp,
          hostname: "demo-laptop",
          os: "macOS 27.2",
          uptime_secs: 412_000 + index * 10,
          cpu: { total_usage: round(cpu, 1), per_core: perCore },
          memory: {
            total_bytes: 32 * 1024 * MiB,
            used_bytes: Math.round((32 * 1024 * MiB * mem) / 100),
            available_bytes: Math.round((32 * 1024 * MiB * (100 - mem)) / 100),
            used_percent: round(mem, 1),
          },
          swap: {
            total_bytes: 8 * 1024 * MiB,
            used_bytes: Math.round((8 * 1024 * MiB * swap) / 100),
            used_percent: round(swap, 1),
          },
          load_average: round(clamp(1.1 + cpu / 22 + (rng() - 0.5) * 0.5, 0.2, 9), 2),
          disks: [
            {
              name: "disk1s5s1",
              mount_point: "/",
              total_bytes: 994 * 1024 * MiB,
              available_bytes: Math.round((994 * 1024 * MiB * (100 - diskA)) / 100),
              used_percent: round(diskA, 1),
            },
            {
              name: "disk1s7s1",
              mount_point: "/System/Volumes/Data",
              total_bytes: 994 * 1024 * MiB,
              available_bytes: Math.round((994 * 1024 * MiB * (100 - diskB)) / 100),
              used_percent: round(diskB, 1),
            },
          ],
          networks: [
            { name: "en0", received_bytes: Math.round(nets.en0.rx), transmitted_bytes: Math.round(nets.en0.tx) },
            { name: "utun0", received_bytes: Math.round(nets.utun0.rx), transmitted_bytes: Math.round(nets.utun0.tx) },
          ],
        },
      }),
    );
  }
  lines.push(footer(count, intervalMs));
  return `${lines.join("\n")}\n`;
}

const PROCESS_DEFS = [
  { pid: 1, name: "launchd", user: "root", cmd: "/sbin/launchd", cpu: 0.3, mem: 34 },
  { pid: 104, name: "kernel_task", user: "root", cmd: "/System/Library/Kernels/kernel", cpu: 6, mem: 1400 },
  { pid: 233, name: "WindowServer", user: "_windowserver", cmd: "/System/Library/PrivateFrameworks/SkyLight.framework/WindowServer", cpu: 9, mem: 900 },
  { pid: 512, name: "mds_stores", user: "root", cmd: "/System/Library/Frameworks/CoreServices.framework/mds_stores", cpu: 4, mem: 480 },
  { pid: 633, name: "mdworker_shared", user: "prakash", cmd: "/System/Library/Frameworks/CoreServices.framework/mdworker_shared", cpu: 2.5, mem: 120 },
  { pid: 777, name: "Safari", user: "prakash", cmd: "/Applications/Safari.app/Contents/MacOS/Safari", cpu: 5, mem: 1300 },
  { pid: 812, name: "com.apple.WebKit.WebContent", user: "prakash", cmd: "/System/Library/Frameworks/WebKit.framework/WebContent", cpu: 7, mem: 700 },
  { pid: 921, name: "Code Helper (Renderer)", user: "prakash", cmd: "/Applications/Visual Studio Code.app/Contents/Frameworks/Code Helper", cpu: 4, mem: 620 },
  { pid: 933, name: "rust-analyzer", user: "prakash", cmd: "/Users/prakash/.rustup/toolchains/stable/bin/rust-analyzer", cpu: 8, mem: 950 },
  { pid: 1005, name: "cargo", user: "prakash", cmd: "cargo build --release --locked", cpu: 3, mem: 300 },
  { pid: 1102, name: "rustc", user: "prakash", cmd: "rustc --crate-name rstats --edition 2021 src/lib.rs", cpu: 12, mem: 620, spike: { from: 0.35, to: 0.62, cpu: 380, mem: 1250 } },
  { pid: 1140, name: "node", user: "prakash", cmd: "node ./scripts/generate-demo.mjs", cpu: 2, mem: 180 },
  { pid: 1210, name: "hermes", user: "prakash", cmd: "hermes gateway run", cpu: 3.5, mem: 560 },
  { pid: 1304, name: "Terminal", user: "prakash", cmd: "/System/Applications/Utilities/Terminal.app", cpu: 1.2, mem: 220 },
  { pid: 1310, name: "zsh", user: "prakash", cmd: "-zsh", cpu: 0.4, mem: 28 },
  { pid: 1402, name: "Dock", user: "prakash", cmd: "/System/Library/CoreServices/Dock.app", cpu: 1.5, mem: 210 },
  { pid: 1408, name: "Finder", user: "prakash", cmd: "/System/Library/CoreServices/Finder.app", cpu: 0.8, mem: 180 },
  { pid: 1510, name: "coreaudiod", user: "_coreaudiod", cmd: "/usr/sbin/coreaudiod", cpu: 1.1, mem: 46 },
  { pid: 1522, name: "bluetoothd", user: "root", cmd: "/usr/sbin/bluetoothd", cpu: 0.6, mem: 24 },
  { pid: 1530, name: "sharingd", user: "prakash", cmd: "/usr/libexec/sharingd", cpu: 0.9, mem: 38 },
  { pid: 1611, name: "cloudd", user: "prakash", cmd: "/System/Library/PrivateFrameworks/CloudKitDaemon.framework/cloudd", cpu: 2.2, mem: 160 },
  { pid: 1622, name: "bird", user: "prakash", cmd: "/System/Library/PrivateFrameworks/CloudDocs.framework/bird", cpu: 1.8, mem: 90 },
  { pid: 1712, name: "configd", user: "root", cmd: "/usr/libexec/configd", cpu: 0.5, mem: 26 },
  { pid: 1718, name: "syslogd", user: "root", cmd: "/usr/sbin/syslogd", cpu: 0.2, mem: 14 },
  { pid: 1804, name: "opendirectoryd", user: "root", cmd: "/usr/libexec/opendirectoryd", cpu: 0.7, mem: 42 },
  { pid: 1812, name: "UserEventAgent", user: "prakash", cmd: "/usr/libexec/UserEventAgent", cpu: 0.6, mem: 52 },
  { pid: 1901, name: "ControlCenter", user: "prakash", cmd: "/System/Library/CoreServices/ControlCenter.app", cpu: 1.4, mem: 190 },
  { pid: 1910, name: "Spotlight", user: "prakash", cmd: "/System/Library/CoreServices/Spotlight.app", cpu: 3, mem: 260, spike: { from: 0.55, to: 0.75, cpu: 60, mem: 420 } },
  { pid: 2004, name: "tmux", user: "prakash", cmd: "tmux new-session -A -s work", cpu: 0.3, mem: 18 },
  { pid: 2101, name: "com.docker.backend", user: "prakash", cmd: "/Applications/Docker.app/Contents/MacOS/com.docker.backend", cpu: 2.8, mem: 720 },
  { pid: 2210, name: "sshd", user: "root", cmd: "/usr/sbin/sshd -i", cpu: 0.2, mem: 12 },
  { pid: 2301, name: "mds", user: "root", cmd: "/System/Library/Frameworks/CoreServices.framework/mds", cpu: 1.6, mem: 110 },
];

function generateDeep() {
  const rng = mulberry32(77);
  const intervalMs = 5_000;
  const count = 97;
  const lines = [header("deep")];
  let disk = 63.1;
  const nets = { en0: { rx: 700 * MiB, tx: 220 * MiB } };
  for (let index = 0; index < count; index += 1) {
    const timestamp = START_MS + index * intervalMs;
    const position = index / (count - 1);
    const processes = [];
    let cpuSum = 0;
    for (const def of PROCESS_DEFS) {
      let cpu = clamp(def.cpu * (0.5 + rng() * 1.6) + (rng() < 0.06 ? 14 * rng() : 0), 0, 100);
      if (def.spike && position >= def.spike.from && position <= def.spike.to) {
        const ramp = Math.sin(Math.PI * ((position - def.spike.from) / (def.spike.to - def.spike.from)));
        cpu = clamp(def.spike.cpu * ramp * (0.7 + 0.6 * rng()), 0, 520);
      }
      const memMb = clamp(def.mem * (0.92 + 0.16 * position) + (rng() - 0.5) * def.mem * 0.08, 8, 4000);
      cpuSum += cpu;
      processes.push({
        pid: def.pid,
        name: def.name,
        user: def.user,
        command: def.cmd,
        cpu_usage: round(cpu, 1),
        memory_bytes: Math.round(memMb * MiB),
        virtual_memory_bytes: Math.round(memMb * MiB * 4),
        runtime_secs: 3600 + index * 5,
        status: "Run",
      });
    }
    const total = clamp(cpuSum / CORES + 4 + (rng() - 0.5) * 3, 2, 99.9);
    const perCore = [];
    for (let core = 0; core < CORES; core += 1) {
      perCore.push(round(clamp(total + (rng() - 0.5) * 30, 0, 100), 1));
    }
    disk = walk(disk, rng, 0.1, 55, 85);
    const rxRate = clamp(260_000 + 1_600_000 * rng() * (0.4 + 0.6 * Math.sin((position * Math.PI) / 1.4)), 40_000, 6_000_000);
    const txRate = clamp(80_000 * (0.5 + rng() * 1.4), 10_000, 1_200_000);
    nets.en0.rx += rxRate * (intervalMs / 1000);
    nets.en0.tx += txRate * (intervalMs / 1000);
    lines.push(
      JSON.stringify({
        type: "sample",
        snapshot: {
          timestamp,
          hostname: "demo-laptop",
          os: "macOS 27.2",
          uptime_secs: 412_000 + index * 5,
          cpu: { total_usage: round(total, 1), per_core: perCore },
          memory: {
            total_bytes: 32 * 1024 * MiB,
            used_bytes: Math.round(32 * 1024 * MiB * 0.58),
            available_bytes: Math.round(32 * 1024 * MiB * 0.42),
            used_percent: 58.2,
          },
          swap: { total_bytes: 8 * 1024 * MiB, used_bytes: Math.round(8 * 1024 * MiB * 0.03), used_percent: 3.1 },
          load_average: round(clamp(1.4 + total / 20, 0.3, 12), 2),
          disks: [
            {
              name: "disk1s5s1",
              mount_point: "/",
              total_bytes: 994 * 1024 * MiB,
              available_bytes: Math.round((994 * 1024 * MiB * (100 - disk)) / 100),
              used_percent: round(disk, 1),
            },
          ],
          networks: [
            { name: "en0", received_bytes: Math.round(nets.en0.rx), transmitted_bytes: Math.round(nets.en0.tx) },
          ],
          processes,
        },
      }),
    );
  }
  lines.push(footer(count, intervalMs));
  return `${lines.join("\n")}\n`;
}

await mkdir(outDir, { recursive: true });
const standard = generateStandard();
const deep = generateDeep();
await writeFile(path.join(outDir, "standard-demo.jsonl"), standard);
await writeFile(path.join(outDir, "deep-demo.jsonl"), deep);
console.log(`standard-demo.jsonl: ${(standard.length / 1024).toFixed(0)} KiB`);
console.log(`deep-demo.jsonl:     ${(deep.length / 1024).toFixed(0)} KiB`);
