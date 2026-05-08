#!/usr/bin/env node
/**
 * recurl npm preuninstall script
 * Removes downloaded binaries.
 */

const fs = require("fs");
const path = require("path");

const binDir = path.join(__dirname, "bin");
const realBinDir = path.join(binDir, "__bin__");

if (fs.existsSync(realBinDir)) {
  try {
    fs.rmSync(realBinDir, { recursive: true, force: true });
    console.log("[recurl] Removed downloaded binaries.");
  } catch (err) {
    console.error(`[recurl] Failed to remove binaries: ${err.message}`);
  }
}
