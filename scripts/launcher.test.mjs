import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

test("launcher maps every supported npm platform", async () => {
  const launcher = await readFile(path.join(root, "npm/rstats-cli/bin/rstats.js"), "utf8");
  for (const packageName of [
    "rstats-cli-darwin-x64",
    "rstats-cli-darwin-arm64",
    "rstats-cli-linux-x64",
    "rstats-cli-linux-arm64",
    "rstats-cli-win32-x64"
  ]) {
    assert.match(launcher, new RegExp(packageName));
  }
  assert.match(launcher, /stdio: "inherit"/);
  assert.doesNotMatch(launcher, /curl|wget|npm install|exec\(/);
});
