"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const { packageRoot } = require("../lib/paths");

const EDITORS = ["cursor", "code", "codium", "code-insiders"];

function installEditor() {
  const extensionDir = path.join(packageRoot(), "editor");
  const manifest = path.join(extensionDir, "package.json");
  if (!fs.existsSync(manifest)) {
    console.error("termvox: bundled editor extension is missing from this package.");
    process.exit(1);
  }

  for (const editor of EDITORS) {
    const result = spawnSync(
      editor,
      ["--install-extension", extensionDir, "--force"],
      { stdio: "pipe", encoding: "utf8" },
    );
    if (result.status === 0) {
      console.log(`termvox: installed editor extension via \`${editor}\``);
      return;
    }
  }

  console.error(
    "termvox: could not install the editor extension automatically.",
  );
  console.error(
    "Install manually: Extensions → Install from Location →",
    extensionDir,
  );
  process.exit(1);
}

if (require.main === module) {
  installEditor();
}

module.exports = { installEditor };
