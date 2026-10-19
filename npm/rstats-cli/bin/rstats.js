#!/usr/bin/env node

const { spawn } = require("node:child_process");
const { existsSync } = require("node:fs");
const path = require("node:path");

const binaries = {
  "darwin-x64": "rstats-darwin-x64",
  "darwin-arm64": "rstats-darwin-arm64",
  "linux-x64": "rstats-linux-x64",
  "linux-arm64": "rstats-linux-arm64",
  "win32-x64": "rstats-win32-x64.exe"
};

const key = `${process.platform}-${process.arch}`;
const binary = binaries[key];

if (!binary) {
  console.error(`rstats-cli does not support ${process.platform}/${process.arch}.`);
  process.exit(1);
}

const executable = path.join(__dirname, binary);
if (!existsSync(executable)) {
  console.error(`The bundled rstats binary for ${key} is missing.`);
  console.error("Reinstall rstats-cli to restore it.");
  process.exit(1);
}

const child = spawn(executable, process.argv.slice(2), { stdio: "inherit" });
child.on("error", (error) => {
  console.error(`Failed to start rstats: ${error.message}`);
  process.exit(1);
});
child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
