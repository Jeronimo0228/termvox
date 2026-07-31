"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const { detectTarget } = require("../lib/platform");
const { releaseTag } = require("../lib/constants");

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
