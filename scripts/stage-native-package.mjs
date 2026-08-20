import { chmod, copyFile, mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = process.argv[2];
const artifact = process.argv[3];
if (!target || !artifact) {
  console.error("Usage: node scripts/stage-native-package.mjs <target> <binary>");
  process.exit(1);
}

const packageNames = {
  "x86_64-apple-darwin": "rstats-cli-darwin-x64",
  "aarch64-apple-darwin": "rstats-cli-darwin-arm64",
  "x86_64-unknown-linux-gnu": "rstats-cli-linux-x64",
  "aarch64-unknown-linux-gnu": "rstats-cli-linux-arm64",
  "x86_64-pc-windows-msvc": "rstats-cli-win32-x64"
};
const packageName = packageNames[target];
if (!packageName) throw new Error(`Unsupported release target: ${target}`);
const executable = target.includes("windows") ? "rstats.exe" : "rstats";
const destination = path.join(root, "npm", packageName, "bin", executable);
await mkdir(path.dirname(destination), { recursive: true });
await copyFile(path.resolve(artifact), destination);
if (!target.includes("windows")) await chmod(destination, 0o755);
console.log(`Staged ${target} binary at ${destination}`);
