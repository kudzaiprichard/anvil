import type { NextConfig } from "next";
import pkg from "./package.json";

const isProd = process.env.NODE_ENV === "production";
// Tauri serves the app from a dev host; the webview loads dev assets from it.
const internalHost = process.env.TAURI_DEV_HOST || "localhost";

const nextConfig: NextConfig = {
  // Tauri ships a static bundle — there is no Node server in production.
  output: "export",
  // The app version shown in the status bar, inlined at build time from
  // package.json (the release script bumps it there), so it can't drift.
  env: {
    NEXT_PUBLIC_APP_VERSION: pkg.version,
  },
  images: {
    unoptimized: true,
  },
  // In dev the Tauri webview loads assets from the Next dev server.
  assetPrefix: isProd ? undefined : `http://${internalHost}:3000`,
  // Keep the dev indicator off the app shell's bottom-left rail buttons.
  devIndicators: {
    position: "bottom-right",
  },
};

export default nextConfig;
