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
      "Fallback: curl -fsSL https://raw.githubusercontent.com/Jeronimo0228/termvox/main/scripts/install.sh | bash",
    );
    process.exit(1);
  }
}

main();
