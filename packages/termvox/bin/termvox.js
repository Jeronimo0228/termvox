#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const { binaryPath } = require("../lib/paths");

function main() {
  const executable = binaryPath();
  if (!fs.existsSync(executable)) {
    console.error(
      "termvox: native binary not installed. Re-run: npm install -g termvox",
    );
    console.error(
      "Or set TERMVOX_SKIP_BINARY_INSTALL=0 and npm rebuild termvox",
    );
    process.exit(1);
  }

  const result = spawnSync(executable, process.argv.slice(2), {
    stdio: "inherit",
    env: process.env,
  });

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  process.exit(result.status ?? 1);
}

main();
