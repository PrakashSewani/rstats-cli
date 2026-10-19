import { access, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const requireBinaries = process.argv.includes("--require-binaries");
const entry = {
  directory: "npm/rstats-cli",
  name: "rstats-cli",
  binary: "bin/rstats.js",
  files: [
    "bin/rstats.js",
    "bin/rstats-darwin-x64",
    "bin/rstats-darwin-arm64",
    "bin/rstats-linux-x64",
    "bin/rstats-linux-arm64",
    "bin/rstats-win32-x64.exe",
    "README.md",
    "LICENSE"
  ],
  nativeBinaries: [
    "bin/rstats-darwin-x64",
    "bin/rstats-darwin-arm64",
    "bin/rstats-linux-x64",
    "bin/rstats-linux-arm64",
    "bin/rstats-win32-x64.exe"
  ]
};

const packageRoot = path.join(root, entry.directory);
const manifest = JSON.parse(await readFile(path.join(packageRoot, "package.json"), "utf8"));
if (manifest.name !== entry.name) throw new Error(`${entry.directory}: incorrect package name`);
if (manifest.bin?.rstats !== entry.binary) throw new Error(`${entry.directory}: incorrect binary mapping`);
if (manifest.optionalDependencies) throw new Error(`${entry.directory}: unexpected optionalDependencies`);
if (manifest.publishConfig?.access !== "public") throw new Error(`${entry.directory}: publish access must be public`);
if (!Array.isArray(manifest.files) || JSON.stringify([...manifest.files].sort()) !== JSON.stringify([...entry.files].sort())) {
  throw new Error(`${entry.directory}: unexpected files allowlist`);
}
await access(path.join(packageRoot, entry.binary));
await access(path.join(packageRoot, "README.md"));
await access(path.join(packageRoot, "LICENSE"));
if (requireBinaries) {
  for (const binary of entry.nativeBinaries) {
    await access(path.join(packageRoot, binary));
  }
}

console.log(`Validated ${entry.name} at version ${manifest.version}${requireBinaries ? " with native binaries" : ""}`);
