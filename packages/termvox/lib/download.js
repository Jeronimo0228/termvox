"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const path = require("node:path");
const { pipeline } = require("node:stream/promises");
const { createWriteStream } = require("node:fs");

/**
 * @param {string} url
 * @returns {Promise<Buffer>}
 */
function fetchBuffer(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "termvox-npm-installer" } }, (response) => {
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          fetchBuffer(response.headers.location).then(resolve, reject);
          return;
        }
        if (response.statusCode !== 200) {
          reject(new Error(`HTTP ${response.statusCode} for ${url}`));
          return;
        }
        const chunks = [];
        response.on("data", (chunk) => chunks.push(chunk));
        response.on("end", () => resolve(Buffer.concat(chunks)));
        response.on("error", reject);
      })
      .on("error", reject);
  });
}

/**
 * @param {string} url
 * @param {string} destination
 */
async function downloadFile(url, destination) {
  await fs.promises.mkdir(path.dirname(destination), { recursive: true });
  await new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "termvox-npm-installer" } }, (response) => {
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          downloadFile(response.headers.location, destination).then(resolve, reject);
          return;
        }
        if (response.statusCode !== 200) {
          reject(new Error(`HTTP ${response.statusCode} for ${url}`));
          return;
        }
        pipeline(response, createWriteStream(destination)).then(resolve, reject);
      })
      .on("error", reject);
  });
}

/**
 * @param {string} filePath
 * @param {string} expectedHash line from .sha256 file
 */
async function verifySha256(filePath, expectedHash) {
  const hash = crypto.createHash("sha256");
  await pipeline(fs.createReadStream(filePath), hash);
  const digest = hash.digest("hex");
  const expected = expectedHash.trim().split(/\s+/)[0];
  if (digest !== expected) {
    throw new Error(`checksum mismatch for ${filePath}`);
  }
}

module.exports = {
  fetchBuffer,
  downloadFile,
  verifySha256,
};
