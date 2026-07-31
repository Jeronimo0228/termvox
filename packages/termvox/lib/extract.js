"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

/**
 * @param {string} archivePath
 * @param {string} destinationDir
 * @param {string} target
 */
async function extractArchive(archivePath, destinationDir, target) {
  await fs.promises.mkdir(destinationDir, { recursive: true });
  const isZip = archivePath.endsWith(".zip");
  if (isZip) {
    if (process.platform === "win32") {
      const result = spawnSync(
        "powershell",
        [
          "-NoProfile",
          "-Command",
          `Expand-Archive -Path '${archivePath.replace(/'/g, "''")}' -DestinationPath '${destinationDir.replace(/'/g, "''")}' -Force`,
        ],
        { stdio: "inherit" },
      );
      if (result.status !== 0) {
        throw new Error("failed to extract zip archive");
      }
    } else {
      const result = spawnSync("unzip", ["-oq", archivePath, "-d", destinationDir], {
        stdio: "inherit",
      });
      if (result.status !== 0) {
        throw new Error("failed to extract zip archive");
      }
    }
    return;
  }

  const result = spawnSync("tar", ["-xzf", archivePath, "-C", destinationDir], {
    stdio: "inherit",
  });
  if (result.status !== 0) {
    throw new Error("failed to extract tar.gz archive");
  }
}

/**
 * @param {string} extractedDir
 * @param {string} binaryPath
 * @param {string} target
 */
async function moveBinary(extractedDir, destBinaryPath, target) {
  const fileName = target.includes("windows") ? "termvox.exe" : "termvox";
  const source = path.join(extractedDir, fileName);
  const resolvedSource = path.resolve(source);
  const resolvedRoot = path.resolve(extractedDir);
  if (
    resolvedSource !== resolvedRoot &&
    !resolvedSource.startsWith(`${resolvedRoot}${path.sep}`)
  ) {
    throw new Error("archive path traversal blocked");
  }
  if (!fs.existsSync(resolvedSource)) {
    throw new Error(`expected binary missing in archive: ${fileName}`);
  }
  const resolvedDest = path.resolve(destBinaryPath);
  await fs.promises.mkdir(path.dirname(resolvedDest), { recursive: true });
  await fs.promises.copyFile(resolvedSource, resolvedDest);
  if (process.platform !== "win32") {
    await fs.promises.chmod(resolvedDest, 0o755);
  }
}

module.exports = {
  extractArchive,
  moveBinary,
};
