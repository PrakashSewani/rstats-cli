import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { fmtElapsed } from "./format";

export interface SeriesSpec {
  label: string;
  color: string;
  width?: number;
  dash?: number[];
  fillAlpha?: number;
}

export interface ChartConfig {
  series: SeriesSpec[];
  x: number[];
  data: (number | null)[][];
  yValues: (values: number[]) => string[];
  height?: number;
  onCursor?: (index: number | null) => void;
}

const AXIS_STROKE = "#55677f";
const GRID_STROKE = "rgba(85, 103, 127, 0.16)";
const FONT = "11.5px ui-monospace, SFMono-Regular, Menlo, monospace";

const syncKeys = new Map<string, { key: string }>();

function syncFor(name: string): { key: string } {
  let sync = syncKeys.get(name);
  if (!sync) {
    const created = uPlot.sync(name);
    sync = { key: created.key };
    syncKeys.set(name, sync);
  }
  return sync;
}

function withAlpha(hex: string, alpha: number): string {
  const value = hex.replace("#", "");
  const red = Number.parseInt(value.slice(0, 2), 16);
  const green = Number.parseInt(value.slice(2, 4), 16);
  const blue = Number.parseInt(value.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}

export interface ChartHandle {
  plot: uPlot;
  dispose: () => void;
}

export function makeChart(mount: HTMLElement, config: ChartConfig, syncName = "rstats-viewer"): ChartHandle {
  const sync = syncFor(syncName);
  const height = config.height ?? 220;
  const options: uPlot.Options = {
    width: mount.clientWidth,
    height,
    padding: [10, 14, 0, 0],
    legend: { show: true, live: true },
    cursor: { sync: { key: sync.key }, y: false, points: { size: 5 } },
    scales: { x: { time: false } },
    axes: [
      {
        stroke: AXIS_STROKE,
        grid: { stroke: GRID_STROKE },
        ticks: { stroke: GRID_STROKE, width: 1 },
        font: FONT,
        values: (_u, values) => values.map((value) => fmtElapsed(value)),
      },
      {
        stroke: AXIS_STROKE,
        grid: { stroke: GRID_STROKE },
        ticks: { stroke: GRID_STROKE, width: 1 },
        font: FONT,
        size: 64,
        values: (_u, values) => config.yValues(values),
      },
    ],
    series: [
      {
        label: "elapsed",
        value: (_u, value) => (value == null ? "" : fmtElapsed(value)),
      },
      ...config.series.map((spec) => ({
        label: spec.label,
        stroke: spec.color,
        width: spec.width ?? 1.7,
        dash: spec.dash,
        fill: spec.fillAlpha ? withAlpha(spec.color, spec.fillAlpha) : undefined,
        value: (_u: uPlot, value: number | null) => (value == null ? "" : config.yValues([value])[0]),
      })),
    ],
    hooks: {
      setCursor: [
        (plot) => {
          const index = plot.cursor.idx;
          config.onCursor?.(index == null ? null : index);
        },
      ],
    },
  };
  const plot = new uPlot(options, [config.x, ...config.data], mount);
  const resize = () => {
    plot.setSize({ width: mount.clientWidth, height });
  };
  const observer = new ResizeObserver(resize);
  observer.observe(mount);
  return {
    plot,
    dispose: () => {
      observer.disconnect();
      plot.destroy();
    },
  };
}

export function resetZoom(handles: ChartHandle[], x: number[]): void {
  if (x.length === 0) return;
  const min = x[0];
  const max = x[x.length - 1];
  for (const handle of handles) {
    handle.plot.setScale("x", { min, max });
  }
}
