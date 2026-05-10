#!/usr/bin/env node
/**
 * recurl npm postinstall script
 * Downloads the correct platform binary from GitHub Releases.
 */

const https = require("https");
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

const VERSION = "0.1.2";
const GITHUB_REPO = "neul-labs/recurl";

function detectPlatform() {
  const platform = process.platform;
  const arch = process.arch;

  const platformMap = {
    darwin: "darwin",
    linux: "linux",
    win32: "windows",
  };

  const archMap = {
    x64: "x86_64",
    arm64: "aarch64",
  };

  const plat = platformMap[platform];
  const cpu = archMap[arch];

  if (!plat || !cpu) {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. recurl supports: darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64`
    );
  }

  return { platform: plat, arch: cpu };
}

function getAssetName(platform, arch) {
  const ext = platform === "windows" ? "zip" : "tar.gz";
  return `recurl-${platform}-${arch}.${ext}`;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, { headers: { "User-Agent": "recurl-npm-installer" } }, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          // Follow redirect
          downloadFile(res.headers.location, dest).then(resolve).catch(reject);
          file.destroy();
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`Download failed: ${res.statusCode} for ${url}`));
          file.destroy();
          return;
        }
        res.pipe(file);
        file.on("finish", () => {
          file.close();
          resolve();
        });
      })
      .on("error", (err) => {
        file.destroy();
        reject(err);
      });
  });
}

function extractTarGz(archivePath, destDir) {
  execSync(`tar -xzf "${archivePath}" -C "${destDir}" --strip-components=1`, {
    stdio: "inherit",
  });
}

function extractZip(archivePath, destDir) {
  execSync(`unzip -o "${archivePath}" -d "${destDir}"`, { stdio: "inherit" });
  // The zip contains a top-level folder; move contents up
  const entries = fs.readdirSync(destDir);
  const topDir = entries.find((e) => fs.statSync(path.join(destDir, e)).isDirectory());
  if (topDir) {
    const topPath = path.join(destDir, topDir);
    const inner = fs.readdirSync(topPath);
    for (const item of inner) {
      fs.renameSync(path.join(topPath, item), path.join(destDir, item));
    }
    fs.rmdirSync(topPath);
  }
}

async function main() {
  const binDir = path.join(__dirname, "bin");
  const realBinDir = path.join(binDir, "__bin__");

  // If __bin__ already exists, assume installed
  if (fs.existsSync(realBinDir)) {
    console.log("[recurl] Binary already installed.");
    return;
  }

  const { platform, arch } = detectPlatform();
  const assetName = getAssetName(platform, arch);
  const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${assetName}`;

  console.log(`[recurl] Downloading ${assetName} for ${platform}-${arch}...`);

  const tmpDir = fs.mkdtempSync(path.join(__dirname, "tmp-"));
  const archivePath = path.join(tmpDir, assetName);

  try {
    await downloadFile(downloadUrl, archivePath);
    console.log("[recurl] Extracting...");

    fs.mkdirSync(realBinDir, { recursive: true });

    if (assetName.endsWith(".tar.gz")) {
      extractTarGz(archivePath, realBinDir);
    } else {
      extractZip(archivePath, realBinDir);
    }

    // Make binaries executable on Unix
    if (platform !== "windows") {
      const binaries = ["recurl", "recurld"];
      for (const bin of binaries) {
        const binPath = path.join(realBinDir, bin);
        if (fs.existsSync(binPath)) {
          fs.chmodSync(binPath, 0o755);
        }
      }
    }

    console.log("[recurl] Installation complete.");
  } catch (err) {
    console.error(`[recurl] Installation failed: ${err.message}`);
    process.exit(1);
  } finally {
    // Cleanup temp directory
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  }
}

main();
