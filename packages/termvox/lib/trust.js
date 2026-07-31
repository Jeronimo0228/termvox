"use strict";

/** @param {string} hostname */
function isAllowedHost(hostname) {
  if (hostname === "github.com") {
    return true;
  }
  return (
    hostname.endsWith(".githubusercontent.com") ||
    hostname === "objects.githubusercontent.com" ||
    hostname === "release-assets.githubusercontent.com"
  );
}

/** @param {URL} parsed */
function isOfficialGitHubReleasePath(parsed) {
  return /^\/Jeronimo0228\/termvox\/releases\/download\/v[^/]+\/termvox-v[^/]+\.(tar\.gz|zip)(\.sha256)?$/.test(
    parsed.pathname,
  );
}

/**
 * Validate the first-hop GitHub release URL (before CDN redirects).
 * @param {string} url
 */
function assertInitialReleaseUrl(url) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    throw new Error("invalid download URL");
  }
  if (parsed.protocol !== "https:") {
    throw new Error("downloads must use HTTPS");
  }
  if (parsed.hostname !== "github.com") {
    throw new Error(`initial download must target github.com, got ${parsed.hostname}`);
  }
  if (!isOfficialGitHubReleasePath(parsed)) {
    throw new Error(`blocked download path: ${parsed.pathname}`);
  }
}

/**
 * Validate redirect targets to GitHub-owned CDNs only.
 * @param {string} url
 */
function assertRedirectUrl(url) {
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    throw new Error("invalid redirect URL");
  }
  if (parsed.protocol !== "https:") {
    throw new Error("downloads must use HTTPS");
  }
  if (!isAllowedHost(parsed.hostname) || parsed.hostname === "github.com") {
    throw new Error(`blocked redirect host: ${parsed.hostname}`);
  }
}

module.exports = {
  assertInitialReleaseUrl,
  assertRedirectUrl,
  isAllowedHost,
  isOfficialGitHubReleasePath,
};
