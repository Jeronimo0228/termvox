"use strict";

const { installBinary } = require("../lib/install-binary");
const { bootstrap } = require("./bootstrap");

async function main() {
  if (process.env.TERMVOX_SKIP_BINARY_INSTALL === "1") {
    console.log("termvox: skipping native binary install (TERMVOX_SKIP_BINARY_INSTALL=1)");
    return;
  }

  try {
    await installBinary();
    bootstrap();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`termvox: install failed — ${message}`);
    console.error(
      "See https://github.com/Jeronimo0228/termvox/blob/main/docs/npm.md#security",
    );
    process.exit(1);
  }
}

main();
