import { access, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const requireBinaries = process.argv.includes("--require-binaries");
const packages = [
  {
    directory: "npm/rstats-cli",
    name: "rstats-cli",
    binary: "bin/rstats.js",
    files: ["bin/rstats.js", "README.md", "LICENSE"]
  },
  {
    directory: "npm/rstats-cli-darwin-x64",
    name: "rstats-cli-darwin-x64",
    binary: "bin/rstats",
    os: "darwin",
    cpu: "x64",
    files: ["bin/rstats", "LICENSE"]
  },
  {
    directory: "npm/rstats-cli-darwin-arm64",
    name: "rstats-cli-darwin-arm64",
    binary: "bin/rstats",
    os: "darwin",
    cpu: "arm64",
    files: ["bin/rstats", "LICENSE"]
  },
  {
    directory: "npm/rstats-cli-linux-x64",
    name: "rstats-cli-linux-x64",
    binary: "bin/rstats",
    os: "linux",
    cpu: "x64",
    files: ["bin/rstats", "LICENSE"]
  },
  {
    directory: "npm/rstats-cli-linux-arm64",
    name: "rstats-cli-linux-arm64",
    binary: "bin/rstats",
    os: "linux",
    cpu: "arm64",
    files: ["bin/rstats", "LICENSE"]
  },
  {
    directory: "npm/rstats-cli-win32-x64",
    name: "rstats-cli-win32-x64",
    binary: "bin/rstats.exe",
    os: "win32",
    cpu: "x64",
    files: ["bin/rstats.exe", "LICENSE"]
  }
];

const readManifest = async (entry) => JSON.parse(await readFile(path.join(root, entry.directory, "package.json"), "utf8"));
const manifests = await Promise.all(packages.map(readManifest));
const versions = new Set(manifests.map((manifest) => manifest.version));
if (versions.size !== 1) throw new Error("All npm package versions must match");

const main = manifests[0];
const expectedOptional = packages.slice(1).map(({ name }) => name).sort();
const actualOptional = Object.keys(main.optionalDependencies ?? {}).sort();
if (JSON.stringify(actualOptional) !== JSON.stringify(expectedOptional)) {
  throw new Error("rstats-cli optionalDependencies do not match platform packages");
}
for (const dependency of expectedOptional) {
  if (main.optionalDependencies[dependency] !== main.version) {
    throw new Error(`${dependency}: optional dependency version does not match rstats-cli`);
  }
}

for (const [index, entry] of packages.entries()) {
  const manifest = manifests[index];
  if (manifest.name !== entry.name) throw new Error(`${entry.directory}: incorrect package name`);
  if (!Array.isArray(manifest.files) || JSON.stringify([...manifest.files].sort()) !== JSON.stringify([...entry.files].sort())) {
    throw new Error(`${entry.directory}: unexpected files allowlist`);
  }
  if (manifest.bin?.rstats !== entry.binary) throw new Error(`${entry.directory}: incorrect binary mapping`);
  if (entry.os && (manifest.os?.length !== 1 || manifest.os[0] !== entry.os)) throw new Error(`${entry.directory}: incorrect os metadata`);
  if (entry.cpu && (manifest.cpu?.length !== 1 || manifest.cpu[0] !== entry.cpu)) throw new Error(`${entry.directory}: incorrect cpu metadata`);
  await access(path.join(root, entry.directory, "package.json"));
  if (index === 0) {
    await access(path.join(root, entry.directory, "bin/rstats.js"));
    await access(path.join(root, entry.directory, "README.md"));
    await access(path.join(root, entry.directory, "LICENSE"));
  } else if (requireBinaries) {
    await access(path.join(root, entry.directory, entry.binary));
  }
}

console.log(`Validated ${packages.length} npm package manifests at version ${manifests[0].version}`);
