import "./styles.css";
import { startRouter } from "./router";
import { renderLanding } from "./pages/landing";
import { renderView } from "./pages/view";

const app = document.getElementById("app");
if (!app) throw new Error("missing #app root");

let dispose: (() => void) | null = null;

startRouter((route) => {
  dispose?.();
  dispose = null;
  app.replaceChildren();
  dispose = route === "view" ? renderView(app) : renderLanding(app);
  window.scrollTo({ top: 0 });
});
