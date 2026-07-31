"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const path = require("node:path");
const { pipeline } = require("node:stream/promises");
const { createWriteStream } = require("node:fs");
const { assertInitialReleaseUrl, assertRedirectUrl } = require("./trust");

const USER_AGENT = "termvox-npm-installer/0.1.0";
const MAX_REDIRECTS = 5;

/**
 * @param {string} url
 * @param {number} redirects
 * @returns {Promise<import("node:http").IncomingMessage>}
 */
function httpsGet(url, redirects = 0) {
  if (redirects === 0) {
    assertInitialReleaseUrl(url);
  } else {
    assertRedirectUrl(url);
  }
  if (redirects > MAX_REDIRECTS) {
    return Promise.reject(new Error("too many redirects"));
  }

  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": USER_AGENT } }, (response) => {
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          const next = new URL(response.headers.location, url).toString();
          httpsGet(next, redirects + 1).then(resolve, reject);
          return;
        }
        if (response.statusCode !== 200) {
          reject(new Error(`HTTP ${response.statusCode} for ${url}`));
          return;
        }
        resolve(response);
      })
      .on("error", reject);
  });
}

/**
 * @param {string} url
 * @returns {Promise<Buffer>}
 */
async function fetchBuffer(url) {
  const response = await httpsGet(url);
  const chunks = [];
  await new Promise((resolve, reject) => {
    response.on("data", (chunk) => chunks.push(chunk));
    response.on("end", resolve);
    response.on("error", reject);
  });
  return Buffer.concat(chunks);
}

/**
 * @param {string} url
 * @param {string} destination
 */
async function downloadFile(url, destination) {
  await fs.promises.mkdir(path.dirname(destination), { recursive: true });
  const response = await httpsGet(url);
  await pipeline(response, createWriteStream(destination));
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
  if (!/^[a-f0-9]{64}$/i.test(expected)) {
    throw new Error("checksum file has invalid format");
  }
  if (digest.toLowerCase() !== expected.toLowerCase()) {
    throw new Error(`checksum mismatch for ${path.basename(filePath)}`);
  }
}

module.exports = {
  downloadFile,
  fetchBuffer,
  verifySha256,
};
