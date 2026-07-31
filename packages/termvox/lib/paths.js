"use strict";

const path = require("node:path");

/** @returns {string} */
function packageRoot() {
  return path.join(__dirname, "..");
}

/** @returns {string} */
function vendorDir() {
  return path.join(packageRoot(), "vendor");
}

/** @returns {string} */
function versionFile() {
  return path.join(vendorDir(), "version.txt");
}

/** @returns {string} */
function binaryName() {
  return process.platform === "win32" ? "termvox.exe" : "termvox";
}

/** @returns {string} */
function binaryPath() {
  return path.join(vendorDir(), binaryName());
}

module.exports = {
  packageRoot,
  vendorDir,
  versionFile,
  binaryName,
  binaryPath,
};
