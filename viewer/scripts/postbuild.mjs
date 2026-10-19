import { writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dist = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "dist");
await writeFile(path.join(dist, "_redirects"), "/* /index.html 200\n");
console.log("postbuild: dist/_redirects written for SPA route fallback");
