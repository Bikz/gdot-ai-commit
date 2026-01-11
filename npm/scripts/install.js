#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execFileSync } = require('child_process');

const REPO = 'Bikz/git-ai-commit';
const BIN_NAME = 'git-ai-commit';

function fail(message) {
  console.error(`error: ${message}`);
  process.exit(1);
}

function resolveTarget() {
  const platform = process.platform;
  const arch = process.arch;

  const archMap = {
    x64: 'x86_64',
    arm64: 'aarch64'
  };

  const osMap = {
    darwin: 'apple-darwin',
    linux: 'unknown-linux-gnu'
  };

  if (!archMap[arch]) {
    fail(`unsupported architecture: ${arch}`);
  }

  if (!osMap[platform]) {
    fail(`unsupported platform: ${platform}`);
  }

  return `${archMap[arch]}-${osMap[platform]}`;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode !== 200) {
        reject(new Error(`status:${response.statusCode}`));
        return;
      }

      response.pipe(file);
      file.on('finish', () => file.close(resolve));
    }).on('error', (err) => {
      fs.unlink(dest, () => reject(err));
    });
  });
}

async function install() {
  const target = resolveTarget();
  const asset = `${BIN_NAME}-${target}.tar.gz`;
  const url = `https://github.com/${REPO}/releases/latest/download/${asset}`;

  const packageRoot = path.join(__dirname, '..');
  const binDir = path.join(packageRoot, 'bin');
  const nativeDir = path.join(binDir, 'native');
  const archivePath = path.join(binDir, asset);
  const binaryPath = path.join(nativeDir, BIN_NAME);

  fs.mkdirSync(nativeDir, { recursive: true });

  if (fs.existsSync(binaryPath)) {
    return;
  }

  console.log(`Downloading ${url}`);
  try {
    await download(url, archivePath);
  } catch (err) {
    if (String(err.message || err).includes('status:404')) {
      fail(
        `no prebuilt binary for ${target}. Build from source: cargo build --release`
      );
    }
    throw err;
  }

  try {
    execFileSync('tar', ['-xzf', archivePath, '-C', nativeDir], { stdio: 'inherit' });
  } catch (err) {
    fail('failed to extract archive (tar required)');
  }

  fs.unlinkSync(archivePath);
  fs.chmodSync(binaryPath, 0o755);
}

install().catch((err) => fail(err.message));
