import { parseFile, ParseError, parseText } from "../lib/parse";
import { BASE, el, LOGO_MARKUP, REPO_URL } from "../lib/dom";
import { navigate } from "../router";
import { setSession } from "../state";

export function renderLanding(root: HTMLElement): () => void {
  root.append(brandBar(), hero(), cards(), steps(), footer());
  return () => {};
}

function brandBar(): HTMLElement {
  const bar = el("div", "topbar");
  const brand = el("a", "brand");
  brand.href = BASE;
  brand.innerHTML = `${LOGO_MARKUP}<span>rstats <span class="crumb">viewer</span></span>`;
  brand.addEventListener("click", (event) => {
    event.preventDefault();
  });
  bar.append(brand);
  return bar;
}

function hero(): HTMLElement {
  const section = el("section", "hero");

  const eyebrow = el("span", "eyebrow");
  eyebrow.append(el("span", "pulse"), el("span", undefined, "100% client-side · no uploads"));
  section.append(eyebrow);

  const title = el("h1");
  title.innerHTML = `Your system recordings, <span class="depth">with depth.</span>`;
  section.append(title);

  section.append(
    el(
      "p",
      "sub",
      "Open an rstats recording and explore CPU, memory, network, and disk history on a zoomable timeline — plus process-level detail from deep captures.",
    ),
  );

  const zone = el("div", "dropzone");
  const heading = el("h2", undefined, "Drop a recording here");
  const hint = el("p", "hint");
  hint.innerHTML =
    "Accepts rstats recordings (<code>.jsonl</code>) and <code>--export --format json</code> output.";
  const choose = el("button", "btn primary", "Choose a file…");
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".jsonl,.json,application/json,text/plain";
  input.hidden = true;
  const status = el(
    "p",
    "privacy",
    "Processed entirely in your browser — your files never leave this machine.",
  );
  zone.append(heading, hint, choose, status, input);

  choose.addEventListener("click", () => input.click());
  input.addEventListener("change", () => {
    const file = input.files?.[0];
    if (file) void loadFile(file, zone, status, section);
    input.value = "";
  });
  zone.addEventListener("dragover", (event) => {
    event.preventDefault();
    zone.classList.add("over");
  });
  zone.addEventListener("dragleave", () => zone.classList.remove("over"));
  zone.addEventListener("drop", (event) => {
    event.preventDefault();
    zone.classList.remove("over");
    const file = event.dataTransfer?.files?.[0];
    if (file) void loadFile(file, zone, status, section);
  });
  section.append(zone);

  const demoRow = el("div", "demo-row");
  demoRow.append(el("span", undefined, "No recording handy?"));
  const standardDemo = el("button", "btn ghost", "Try a standard demo");
  standardDemo.addEventListener("click", () => void loadDemo("standard", zone, status, section));
  const deepDemo = el("button", "btn ghost", "Try a deep demo (process data)");
  deepDemo.addEventListener("click", () => void loadDemo("deep", zone, status, section));
  demoRow.append(standardDemo, el("span", "faint", "or"), deepDemo);
  section.append(demoRow);

  return section;
}

async function loadFile(
  file: File,
  zone: HTMLElement,
  status: HTMLElement,
  container: HTMLElement,
): Promise<void> {
  setBusy(zone, status, `Reading ${file.name}…`);
  clearError(container);
  try {
    const session = await parseFile(file);
    setSession(session);
    navigate("view");
  } catch (error) {
    zone.classList.remove("busy");
    status.textContent =
      "Processed entirely in your browser — your files never leave this machine.";
    showError(container, error instanceof ParseError ? error.message : "Could not read this file.");
  }
}

async function loadDemo(
  kind: "standard" | "deep",
  zone: HTMLElement,
  status: HTMLElement,
  container: HTMLElement,
): Promise<void> {
  setBusy(zone, status, `Loading ${kind} demo…`);
  clearError(container);
  try {
    const response = await fetch(`${BASE}demo/${kind}-demo.jsonl`);
    if (!response.ok) throw new Error(`demo fetch failed: ${response.status}`);
    const text = await response.text();
    const session = parseText(text, `${kind}-demo.jsonl`);
    setSession(session);
    navigate("view");
  } catch {
    zone.classList.remove("busy");
    status.textContent =
      "Processed entirely in your browser — your files never leave this machine.";
    showError(container, "Could not load the demo recording. Try opening your own file instead.");
  }
}

function setBusy(zone: HTMLElement, status: HTMLElement, message: string): void {
  zone.classList.add("busy");
  status.replaceChildren(el("span", "spinner"), document.createTextNode(message));
}

function clearError(container: HTMLElement): void {
  container.querySelector(".error-box")?.remove();
}

function showError(container: HTMLElement, message: string): void {
  const box = el("div", "error-box", message);
  container.append(box);
}

function cards(): HTMLElement {
  const grid = el("div", "cards");
  const items: Array<{ icon: string; tone: string; title: string; body: string }> = [
    {
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19V5"/><path d="M4 19h16"/><path d="M7 15l4-5 3 3 5-7"/></svg>`,
      tone: "cyan",
      title: "View with depth",
      body: "Per-core CPU, memory, swap, load, per-interface network rates, and worst-disk usage — aligned on one time axis, zoomable and pannable.",
    },
    {
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="7" y="7" width="10" height="10" rx="2"/><path d="M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3"/></svg>`,
      tone: "green",
      title: "Process forensics",
      body: "Deep captures include the process table — scrub the timeline and see exactly which processes were running hot at any moment.",
    },
    {
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/></svg>`,
      tone: "yellow",
      title: "Private by design",
      body: "Files are parsed locally in your browser. Nothing is uploaded, stored, or sent anywhere — bring recordings from any machine.",
    },
    {
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3l9 5-9 5-9-5 9-5z"/><path d="M3 13.5l9 5 9-5"/></svg>`,
      tone: "purple",
      title: "Every capture, portable",
      body: "Standard and deep recordings, raw .jsonl sessions, or flattened --export output — from macOS, Linux, or Windows.",
    },
  ];
  for (const item of items) {
    const card = el("div", "card");
    const glyph = el("span", `glyph ${item.tone}`);
    glyph.innerHTML = item.icon;
    card.append(glyph, el("h3", undefined, item.title), el("p", undefined, item.body));
    grid.append(card);
  }
  return grid;
}

function steps(): HTMLElement {
  const section = el("section", "steps");
  section.append(el("h2", undefined, "How to capture a recording"));
  const list = el("ol");
  const one = el("li");
  one.innerHTML = `Run <code>rstats</code> and press <code>s</code> to start recording — or <code>S</code> for deep capture with the process table.`;
  const two = el("li");
  two.innerHTML = `Press <code>s</code> again to stop; the file lands in your recordings directory as <code>rstats-*.jsonl</code>.`;
  const three = el("li");
  three.innerHTML = `Drop it above — or convert first with <code>rstats --export recordings/rstats-*.jsonl --format json</code>.`;
  list.append(one, two, three);
  section.append(list);
  return section;
}

function footer(): HTMLElement {
  const bar = el("div", "page-footer");
  const mark = el("span");
  mark.innerHTML = `${LOGO_MARKUP}`;
  bar.append(mark, el("span", undefined, "rstats viewer — recordings with depth"));
  bar.append(el("div", "spacer"));
  const link = el("a", undefined, "rstats-cli on GitHub");
  link.href = REPO_URL;
  link.target = "_blank";
  link.rel = "noreferrer";
  bar.append(link, el("span", undefined, "MIT"));
  return bar;
}
