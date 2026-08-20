#!/usr/bin/env node

const { spawn } = require("node:child_process");
const path = require("node:path");

const packages = {
  "darwin-x64": "rstats-cli-darwin-x64",
  "darwin-arm64": "rstats-cli-darwin-arm64",
  "linux-x64": "rstats-cli-linux-x64",
  "linux-arm64": "rstats-cli-linux-arm64",
  "win32-x64": "rstats-cli-win32-x64"
};

const key = `${process.platform}-${process.arch}`;
const packageName = packages[key];

if (!packageName) {
  console.error(`rstats-cli does not support ${process.platform}/${process.arch}.`);
  process.exit(1);
}

let executable;
try {
  executable = require.resolve(`${packageName}/bin/rstats${process.platform === "win32" ? ".exe" : ""}`);
} catch {
  console.error(`The native rstats binary for ${key} is not installed.`);
  console.error(`Reinstall rstats-cli on a supported platform or install ${packageName} directly.`);
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
