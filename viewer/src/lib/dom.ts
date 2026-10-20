export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

export const REPO_URL = "https://github.com/PrakashSewani/rstats-cli";

export const BASE = import.meta.env.BASE_URL;

export const LOGO_MARKUP = `<svg width="22" height="22" viewBox="0 0 32 32" aria-hidden="true"><rect width="32" height="32" rx="7" fill="#0b1220"/><path d="M5 21 L10 21 L12.5 11 L16 25 L19 15 L21.5 17.5 L27 17.5" fill="none" stroke="#22d3ee" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
