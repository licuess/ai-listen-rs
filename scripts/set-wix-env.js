// Sets TAURI_WIX_PATH environment variable if WiX is cached locally
// This runs before `tauri build` to ensure the MSI bundler finds WiX

import { existsSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

// Tauri v2 uses %LOCALAPPDATA%\tauri\WixTools314 on Windows
const wixDir = join(homedir(), "AppData", "Local", "tauri", "WixTools314");
const candleExe = join(wixDir, "candle.exe");

if (existsSync(candleExe)) {
  process.env.TAURI_WIX_PATH = wixDir;
  console.log(`WiX path set: ${wixDir}`);
} else {
  console.log("WiX not found locally, tauri will attempt to download it.");
  console.log("Run `npm run wix:install` to pre-install WiX and avoid download timeouts.");
}
