import { assetManifestPlugin } from "./scripts/asset-manifest";
import { latestUpdateDate } from "./scripts/build-info";
import { defineConfig } from "vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { nitro } from "nitro/vite";

export default defineConfig(({ command, isPreview }) => ({
  define: { __BLACKSITE_UPDATED_AT__: JSON.stringify(latestUpdateDate()) },
  server: {
    host: "0.0.0.0", port: 8080, strictPort: true,
    // Local QA contains Python environments and model caches, not app sources.
    watch: { ignored: ["**/.blacksite/**"] },
  },
  preview: { host: "127.0.0.1", port: 8081, strictPort: true },
  resolve: { tsconfigPaths: true },
  plugins: [assetManifestPlugin(),
    tailwindcss(),
    tanstackStart(),
    ...(command === "build" || isPreview ? [nitro({ preset: "vercel" })] : []),
    viteReact(),
  ],
}));
