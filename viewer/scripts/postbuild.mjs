import { copyFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dist = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "dist");
await copyFile(path.join(dist, "index.html"), path.join(dist, "404.html"));
console.log("postbuild: dist/404.html written for SPA route fallback");
