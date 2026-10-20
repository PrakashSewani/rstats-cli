import type { Session } from "./lib/types";

let current: Session | null = null;

export function setSession(session: Session): void {
  current = session;
}

export function getSession(): Session | null {
  return current;
}
