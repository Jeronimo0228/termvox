"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const { detectTarget } = require("../lib/platform");
const { releaseTag } = require("../lib/constants");
const {
  assertInitialReleaseUrl,
  assertRedirectUrl,
} = require("../lib/trust");
const { expectedAssetName } = require("../lib/install-binary");

test("detectTarget returns null or a rust triple", () => {
  const target = detectTarget();
  if (target) {
    assert.match(target, /^[a-z0-9_]+-(unknown-linux-gnu|apple-darwin|pc-windows-msvc)$/);
  }
});

test("releaseTag prefixes v when missing", () => {
  assert.equal(releaseTag("0.1.0-alpha.8"), "v0.1.0-alpha.8");
  assert.equal(releaseTag("v0.1.0-alpha.8"), "v0.1.0-alpha.8");
});

test("expectedAssetName matches release archives", () => {
  assert.equal(
    expectedAssetName("0.1.0-alpha.8", "x86_64-unknown-linux-gnu"),
    "termvox-v0.1.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz",
  );
});

test("assertInitialReleaseUrl allows official GitHub release asset", () => {
  assert.doesNotThrow(() =>
    assertInitialReleaseUrl(
      "https://github.com/Jeronimo0228/termvox/releases/download/v0.1.0-alpha.8/termvox-v0.1.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz",
    ),
  );
});

test("assertInitialReleaseUrl blocks third-party hosts", () => {
  assert.throws(
    () =>
      assertInitialReleaseUrl(
        "https://evil.example/Jeronimo0228/termvox/releases/download/v0.1.0-alpha.8/termvox-v0.1.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz",
      ),
    /github.com/,
  );
});

test("assertInitialReleaseUrl blocks wrong repository path", () => {
  assert.throws(
    () =>
      assertInitialReleaseUrl(
        "https://github.com/evil/termvox/releases/download/v0.1.0-alpha.8/termvox-v0.1.0-alpha.8-x86_64-unknown-linux-gnu.tar.gz",
      ),
    /blocked download path/,
  );
});

test("assertRedirectUrl allows GitHub CDN hosts", () => {
  assert.doesNotThrow(() =>
    assertRedirectUrl(
      "https://release-assets.githubusercontent.com/github-production-release-asset/1313967344/64f3c6c1-c3c6-447a-b2f6-5addffd24347",
    ),
  );
});

test("assertRedirectUrl blocks non-GitHub hosts", () => {
  assert.throws(
    () => assertRedirectUrl("https://evil.example/asset.tar.gz"),
    /blocked redirect host/,
  );
});
