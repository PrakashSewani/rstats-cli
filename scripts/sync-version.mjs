import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageFiles = [
  "package.json",
  "npm/rstats-cli/package.json",
  "npm/rstats-cli-darwin-x64/package.json",
  "npm/rstats-cli-darwin-arm64/package.json",
  "npm/rstats-cli-linux-x64/package.json",
  "npm/rstats-cli-linux-arm64/package.json",
  "npm/rstats-cli-win32-x64/package.json"
];
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
  const raw = await readFile(file, "utf8");
  const manifest = JSON.parse(raw);
  let changed = manifest.version !== version;
  if (changed) {
    mismatches.push(`${relative}: ${manifest.version ?? "missing"} (expected ${version})`);
  }
  if (manifest.optionalDependencies) {
    for (const [dependency, dependencyVersion] of Object.entries(manifest.optionalDependencies)) {
      if (dependencyVersion !== version) {
        changed = true;
        mismatches.push(`${relative} optional dependency ${dependency}: ${dependencyVersion} (expected ${version})`);
      }
    }
  }
  if (!checkOnly && changed) {
    manifest.version = version;
    if (manifest.optionalDependencies) {
      for (const dependency of Object.keys(manifest.optionalDependencies)) {
        manifest.optionalDependencies[dependency] = version;
      }
    }
    await writeFile(file, `${JSON.stringify(manifest, null, 2)}\n`);
  }
}

if (checkOnly && mismatches.length) {
  throw new Error(`npm versions are out of sync:\n${mismatches.join("\n")}`);
}
if (!checkOnly) {
  console.log(`Synchronized npm manifests to ${version}`);
}
