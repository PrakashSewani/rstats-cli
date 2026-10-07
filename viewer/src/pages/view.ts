import { buildStats } from "../lib/stats";
import { makeChart, resetZoom, type ChartHandle, type SeriesSpec } from "../lib/charts";
import { el, BASE, LOGO_MARKUP, REPO_URL } from "../lib/dom";
import { fmtBytes, fmtClock, fmtDateTime, fmtDuration, fmtLoad, fmtOffset, fmtPct } from "../lib/format";
import type { ProcessRow, Session } from "../lib/types";
import { navigate } from "../router";
import { getSession } from "../state";

const CORE_COLORS = ["#67e8f9", "#4ade80", "#facc15", "#fb923c", "#a78bfa", "#f472b6", "#38bdf8", "#f87171"];

export function renderView(root: HTMLElement): () => void {
  const session = getSession();
  if (!session) {
    root.append(emptyState());
    return () => {};
  }

  const start = session.timestamps[0] ?? 0;
  const end = session.stoppedAtMs ?? session.timestamps[session.samples - 1] ?? start;
  const x = session.timestamps.map((timestamp) => (timestamp - start) / 1000);
  const stats = buildStats(session);
  const disposers: Array<() => void> = [];
  const charts: ChartHandle[] = [];
  let cursorIndex = session.samples - 1;
  let sliderEl: HTMLInputElement | null = null;
  let updateProcesses: (() => void) | null = null;

  const syncIndicators = () => {
    if (sliderEl && sliderEl.value !== String(cursorIndex)) sliderEl.value = String(cursorIndex);
    updateProcesses?.();
  };

  const onCursor = (index: number | null) => {
    if (index == null) return;
    cursorIndex = Math.min(Math.max(index, 0), session.samples - 1);
    syncIndicators();
  };

  root.append(topbar(session), kpiRow(session, start, end, stats));

  const toolbar = el("div", "chart-head");
  toolbar.append(
    el("h3", undefined, "Timeline"),
    el("span", "hint", "drag across a chart to zoom · double-click to reset · hover to inspect"),
  );
  toolbar.append(el("div", "spacer"));
  const resetButton = el("button", "btn small", "Reset zoom");
  resetButton.addEventListener("click", () => resetZoom(charts, x));
  toolbar.append(resetButton);
  root.append(toolbar);

  const chartsWrap = el("div", "charts");
  const addChart = (
    title: string,
    hint: string,
    config: Parameters<typeof makeChart>[1],
    height = 210,
  ): ChartHandle => {
    const card = el("div", "chart-card");
    const head = el("div", "chart-head");
    head.append(el("h3", undefined, title));
    if (hint) head.append(el("span", "hint", hint));
    const body = el("div", "chart-body");
    card.append(head, body);
    chartsWrap.append(card);
    const handle = makeChart(body, { ...config, height, onCursor });
    charts.push(handle);
    disposers.push(handle.dispose);
    return handle;
  };

  const pctFormat = (values: number[]) => values.map((value) => `${value.toFixed(1)}%`);
  const cpuSeries: SeriesSpec[] = [{ label: "CPU total", color: "#22d3ee", width: 2, fillAlpha: 0.13 }];
  if (session.perCore) {
    session.perCore.forEach((_series, index) => {
      cpuSeries.push({
        label: `core ${index + 1}`,
        color: CORE_COLORS[index % CORE_COLORS.length],
        width: 1,
      });
    });
  }
  addChart("CPU", session.perCore ? "total + per-core" : "total", {
    series: cpuSeries,
    x,
    data: [session.cpu, ...(session.perCore ?? [])],
    yValues: pctFormat,
  }, 250);

  addChart("Memory", "used % · swap %", {
    series: [
      { label: "Memory", color: "#4ade80", width: 2, fillAlpha: 0.12 },
      { label: "Swap", color: "#facc15", width: 1.4 },
    ],
    x,
    data: [session.mem, session.swap],
    yValues: pctFormat,
  });

  addChart("Load average", "1-minute load", {
    series: [{ label: "Load", color: "#fb923c", width: 2, fillAlpha: 0.1 }],
    x,
    data: [session.load],
    yValues: (values) => values.map((value) => value.toFixed(2)),
  }, 180);

  addChart("Network", "receive · transmit", {
    series: [
      { label: "↓ Receive", color: "#38bdf8", width: 1.8, fillAlpha: 0.1 },
      { label: "↑ Transmit", color: "#a78bfa", width: 1.8 },
    ],
    x,
    data: [session.netRx, session.netTx],
    yValues: (values) => values.map((value) => fmtBytes(value, true)),
  }, 190);

  addChart("Disk", "worst-disk usage", {
    series: [{ label: "Worst disk", color: "#f87171", width: 2, fillAlpha: 0.12 }],
    x,
    data: [session.disk],
    yValues: pctFormat,
  }, 180);

  root.append(chartsWrap);

  const seek = (index: number) => {
    cursorIndex = Math.min(Math.max(Math.round(index), 0), session.samples - 1);
    const primary = charts[0]?.plot;
    if (primary) {
      primary.setCursor({
        left: primary.valToPos(x[cursorIndex], "x"),
        top: Math.max(1, primary.bbox.height / 2),
      });
    }
    syncIndicators();
  };

  if (session.processes) {
    const panel = processPanel(session, {
      getIndex: () => cursorIndex,
      onScrub: (index) => seek(index),
    });
    updateProcesses = panel.update;
    sliderEl = panel.slider;
    root.append(panel.element);
  }

  root.append(footerNote());
  syncIndicators();

  return () => {
    for (const dispose of disposers) dispose();
  };
}

function emptyState(): HTMLElement {
  const wrap = el("div", "empty-state");
  wrap.append(el("h2", undefined, "No recording loaded"));
  wrap.append(el("p", undefined, "Open a recording to explore its timeline here."));
  const button = el("button", "btn primary", "Open a recording");
  button.addEventListener("click", () => navigate("landing"));
  wrap.append(button);
  return wrap;
}

function topbar(session: Session): HTMLElement {
  const bar = el("div", "topbar");
  const brand = el("a", "brand");
  brand.href = BASE;
  brand.innerHTML = `${LOGO_MARKUP}<span>rstats <span class="crumb">viewer</span></span>`;
  brand.addEventListener("click", (event) => {
    event.preventDefault();
    navigate("landing");
  });
  bar.append(brand);

  const meta = el("div", "file-meta");
  meta.append(el("span", "name", session.fileName), scopeBadge(session));
  const bits: string[] = [];
  if (session.hostname) bits.push(session.hostname);
  if (session.os) bits.push(session.os);
  bits.push(fmtDateTime(session.startedAtMs));
  bits.push(fmtDuration(sessionEnd(session) - session.startedAtMs));
  bits.push(`${session.samples.toLocaleString()} samples`);
  meta.append(el("span", undefined, bits.join(" · ")));
  bar.append(meta, el("div", "spacer"));

  const openAnother = el("button", "btn small", "Open another");
  openAnother.addEventListener("click", () => navigate("landing"));
  bar.append(openAnother);
  return bar;
}

function sessionEnd(session: Session): number {
  return session.stoppedAtMs ?? session.timestamps[session.samples - 1] ?? session.startedAtMs;
}

function scopeBadge(session: Session): HTMLElement {
  if (session.source === "export") {
    return el("span", "badge export dot", "export · charts only");
  }
  if (session.scope === "deep") {
    return el("span", "badge deep dot", "deep capture · process data");
  }
  return el("span", "badge dot", "standard capture");
}

function kpiRow(
  session: Session,
  start: number,
  end: number,
  stats: ReturnType<typeof buildStats>,
): HTMLElement {
  const grid = el("div", "kpis");
  const card = (label: string, value: string, detail: string) => {
    const box = el("div", "kpi");
    box.append(el("div", "label", label), el("div", "value", value), el("div", "detail", detail));
    return box;
  };
  const peakDetail = (summary: { peak: number | null; peakOffsetMs: number | null }, format: (value: number | null) => string) =>
    `peak ${format(summary.peak)}${summary.peakOffsetMs != null ? ` · ${fmtOffset(summary.peakOffsetMs)}` : ""}`;

  grid.append(
    card("CPU avg", fmtPct(stats.cpu.avg), peakDetail(stats.cpu, fmtPct)),
    card("Memory avg", fmtPct(stats.mem.avg), peakDetail(stats.mem, fmtPct)),
    card("Load avg", fmtLoad(stats.load.avg), peakDetail(stats.load, fmtLoad)),
    card("Network", `↓ ${fmtBytes(stats.receivedTotal)}`, `↑ ${fmtBytes(stats.transmittedTotal)} over session`),
    card("Disk peak", fmtPct(stats.disk.peak), `avg ${fmtPct(stats.disk.avg)}`),
    card("Session", fmtDuration(end - start), `${session.samples.toLocaleString()} samples`),
  );
  return grid;
}

interface ProcessPanelHandlers {
  getIndex: () => number;
  onScrub: (index: number) => void;
}

function processPanel(
  session: Session,
  handlers: ProcessPanelHandlers,
): { element: HTMLElement; update: () => void; slider: HTMLInputElement } {
  const wrap = el("section", "processes");
  const head = el("div", "panel-head");
  head.append(el("h3", undefined, "Processes"));
  const when = el("span", "when", "");
  head.append(when, el("div", "spacer"));

  const tabs = el("div", "sort-tabs");
  const cpuButton = el("button", undefined, "CPU");
  const memButton = el("button", undefined, "Memory");
  tabs.append(cpuButton, memButton);
  head.append(tabs);
  wrap.append(head);

  const scrubRow = el("div", "scrub");
  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = "0";
  slider.max = String(Math.max(0, session.samples - 1));
  slider.value = String(Math.max(0, session.samples - 1));
  slider.addEventListener("input", () => handlers.onScrub(Number(slider.value)));
  const scrubLabel = el("span", undefined, "");
  scrubRow.append(slider, scrubLabel);
  wrap.append(scrubRow);

  const tableWrap = el("div", "process-table-wrap");
  const table = document.createElement("table");
  table.className = "process-table";
  const thead = document.createElement("thead");
  thead.innerHTML =
    "<tr><th>PID</th><th>Process</th><th>User</th><th>CPU %</th><th>Memory</th><th>Command</th></tr>";
  const tbody = document.createElement("tbody");
  table.append(thead, tbody);
  tableWrap.append(table);
  wrap.append(tableWrap);

  let sort: "cpu" | "mem" = "cpu";
  const setActive = () => {
    cpuButton.classList.toggle("active", sort === "cpu");
    memButton.classList.toggle("active", sort === "mem");
  };
  const update = () => {
    const index = handlers.getIndex();
    const frame = session.processes?.[index];
    const rows = frame ? [...frame.top] : [];
    rows.sort((left, right) =>
      sort === "cpu" ? right.cpu - left.cpu : right.memBytes - left.memBytes,
    );
    tbody.replaceChildren(...rows.slice(0, 20).map(rowElement));
    const timestamp = session.timestamps[index] ?? session.timestamps[0] ?? 0;
    const origin = session.timestamps[0] ?? timestamp;
    when.textContent = `${fmtClock(timestamp)} · ${fmtOffset(timestamp - origin)}`;
    scrubLabel.textContent = `sample ${index + 1} / ${session.samples}`;
  };
  cpuButton.addEventListener("click", () => {
    sort = "cpu";
    setActive();
    update();
  });
  memButton.addEventListener("click", () => {
    sort = "mem";
    setActive();
    update();
  });
  setActive();
  update();

  return { element: wrap, update, slider };
}

function rowElement(row: ProcessRow): HTMLElement {
  const tr = document.createElement("tr");
  const cells: Array<{ text: string; className?: string }> = [
    { text: String(row.pid) },
    { text: row.name, className: "name" },
    { text: row.user || "–" },
    { text: fmtPct(row.cpu), className: `cpu${row.cpu >= 100 ? " scorching" : row.cpu >= 30 ? " hot" : ""}` },
    { text: fmtBytes(row.memBytes) },
    { text: row.command ?? "", className: "cmd" },
  ];
  for (const cell of cells) {
    const td = document.createElement("td");
    if (cell.className) td.className = cell.className;
    td.textContent = cell.text;
    td.title = cell.text;
    tr.append(td);
  }
  return tr;
}

function footerNote(): HTMLElement {
  const bar = el("div", "page-footer");
  bar.append(el("span", undefined, "Processed locally in your browser — nothing is uploaded."));
  bar.append(el("div", "spacer"));
  const link = el("a", undefined, "rstats-cli on GitHub");
  link.href = REPO_URL;
  link.target = "_blank";
  link.rel = "noreferrer";
  bar.append(link, el("span", undefined, "MIT"));
  return bar;
}
