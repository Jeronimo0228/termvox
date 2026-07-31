"use strict";

const { readFileSync } = require("node:fs");
const path = require("node:path");

const DEFAULT_REPO = "Jeronimo0228/termvox";

/** @returns {{ version: string, repo: string }} */
function readPackageMeta() {
  const manifest = JSON.parse(
    readFileSync(path.join(__dirname, "..", "package.json"), "utf8"),
  );
  return {
    version: manifest.version,
    repo: process.env.TERMVOX_INSTALL_REPO || DEFAULT_REPO,
  };
}

/** @param {string} version */
function releaseTag(version) {
  return version.startsWith("v") ? version : `v${version}`;
}

module.exports = {
  DEFAULT_REPO,
  readPackageMeta,
  releaseTag,
};
