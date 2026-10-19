import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

test("launcher maps every supported bundled binary", async () => {
  const launcher = await readFile(path.join(root, "npm/rstats-cli/bin/rstats.js"), "utf8");
  for (const key of ["darwin-x64", "darwin-arm64", "linux-x64", "linux-arm64", "win32-x64"]) {
    assert.match(launcher, new RegExp(key));
  }
  for (const binary of [
    "rstats-darwin-x64",
    "rstats-darwin-arm64",
    "rstats-linux-x64",
    "rstats-linux-arm64",
    "rstats-win32-x64.exe"
  ]) {
    assert.match(launcher, new RegExp(binary.replace(".", "\\.")));
  }
  assert.match(launcher, /path\.join\(__dirname, binary\)/);
  assert.match(launcher, /stdio: "inherit"/);
  assert.doesNotMatch(launcher, /curl|wget|npm install|exec\(/);
});
