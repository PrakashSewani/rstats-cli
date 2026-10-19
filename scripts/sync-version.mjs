import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageFiles = ["package.json", "npm/rstats-cli/package.json"];
const checkOnly = process.argv.includes("--check");

const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
const versionMatch = cargo.match(/^version\s*=\s*"([^"]+)"/m);
if (!versionMatch) {
  throw new Error("Could not find package version in Cargo.toml");
}
const version = versionMatch[1];
const mismatches = [];

for (const relative of packageFiles) {
  const file = path.join(root, relative);
  const manifest = JSON.parse(await readFile(file, "utf8"));
  const changed = manifest.version !== version;
  if (changed) {
    mismatches.push(`${relative}: ${manifest.version ?? "missing"} (expected ${version})`);
  }
  if (!checkOnly && changed) {
    manifest.version = version;
    await writeFile(file, `${JSON.stringify(manifest, null, 2)}\n`);
  }
}

if (checkOnly && mismatches.length) {
  throw new Error(`npm versions are out of sync:\n${mismatches.join("\n")}`);
}
if (!checkOnly) {
  console.log(`Synchronized npm manifests to ${version}`);
}
