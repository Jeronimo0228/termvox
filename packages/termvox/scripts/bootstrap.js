"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { binaryPath } = require("../lib/paths");

function configExists() {
  const global = path.join(
    process.env.XDG_CONFIG_HOME || path.join(os.homedir(), ".config"),
    "termvox",
    "termvox.toml",
  );
  const project = path.join(process.cwd(), "termvox.toml");
  return fs.existsSync(global) || fs.existsSync(project);
}

function runTermvox(args) {
  const executable = binaryPath();
  if (!fs.existsSync(executable)) {
    return;
  }
  spawnSync(executable, args, { stdio: "inherit", env: process.env });
}

function bootstrap() {
  if (process.env.TERMVOX_SKIP_BOOTSTRAP === "1") {
    return;
  }

  console.log("termvox: bootstrapping Whisper model and default config...");
  runTermvox(["models", "install", "default"]);

  if (!configExists()) {
    const preset = process.env.TERMVOX_NPM_PRESET || "cursor";
    const init = spawnSync(binaryPath(), ["init", "--preset", preset, "--force"], {
      stdio: "inherit",
      env: process.env,
    });
    if (init.status !== 0) {
      runTermvox(["init", "--force"]);
    }
  }

  console.log("termvox: ready — run `termvox doctor`, then `termvox shell`");
  console.log("termvox: optional editor UI: `termvox-editor-install`");
}

module.exports = { bootstrap };
