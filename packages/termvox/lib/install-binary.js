"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { readPackageMeta, releaseTag } = require("./constants");
const { downloadFile, verifySha256 } = require("./download");
const { extractArchive, moveBinary } = require("./extract");
const { detectTarget } = require("./platform");
const { binaryPath, versionFile } = require("./paths");

/**
 * @param {string} version
 */
function isInstalled(version) {
  try {
    const installed = fs.readFileSync(versionFile(), "utf8").trim();
    return installed === version && fs.existsSync(binaryPath());
  } catch {
    return false;
  }
}

/**
 * @param {string} version
 * @param {string} target
 */
function expectedAssetName(version, target) {
  const extension = target.includes("windows") ? "zip" : "tar.gz";
  return `termvox-v${version}-${target}.${extension}`;
}

async function installBinary() {
  const { version, repo } = readPackageMeta();
  if (isInstalled(version)) {
    console.log(`termvox: native binary already installed (${version})`);
    return binaryPath();
  }

  const target = detectTarget();
  if (!target) {
    throw new Error(
      `unsupported platform: ${process.platform}-${process.arch}. Install from source: https://github.com/${repo}`,
    );
  }

  const tag = releaseTag(version);
  const asset = expectedAssetName(version, target);
  const base = `https://github.com/${repo}/releases/download/${tag}`;
  const tmp = await fs.promises.mkdtemp(path.join(os.tmpdir(), "termvox-npm-"));

  try {
    const archivePath = path.join(tmp, asset);
    console.log(`termvox: downloading ${asset} ...`);
    await downloadFile(`${base}/${asset}`, archivePath);

    const checksumPath = path.join(tmp, `${asset}.sha256`);
    await downloadFile(`${base}/${asset}.sha256`, checksumPath);
    const expected = await fs.promises.readFile(checksumPath, "utf8");
    await verifySha256(archivePath, expected);

    const extractDir = path.join(tmp, "extract");
    await extractArchive(archivePath, extractDir, target);
    await moveBinary(extractDir, binaryPath(), target);
    await fs.promises.mkdir(path.dirname(versionFile()), { recursive: true });
    await fs.promises.writeFile(versionFile(), `${version}\n`, "utf8");
    console.log(`termvox: installed native CLI ${version} for ${target}`);
    return binaryPath();
  } finally {
    await fs.promises.rm(tmp, { recursive: true, force: true });
  }
}

module.exports = {
  installBinary,
  isInstalled,
  expectedAssetName,
};
