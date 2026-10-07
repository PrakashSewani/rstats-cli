export type Route = "landing" | "view";

const base = import.meta.env.BASE_URL;

export function routeFromLocation(): Route {
  const path = location.pathname.startsWith(base)
    ? location.pathname.slice(base.length)
    : location.pathname;
  return path.replace(/^\/+|\/+$/g, "") === "view" ? "view" : "landing";
}

export function navigate(route: Route): void {
  const path = route === "view" ? "view" : "";
  history.pushState(null, "", `${base}${path}`);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

export function startRouter(render: (route: Route) => void): void {
  window.addEventListener("popstate", () => render(routeFromLocation()));
  render(routeFromLocation());
}
