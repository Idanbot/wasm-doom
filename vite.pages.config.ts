import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

/** Explicit base wins; Pages workflows inherit their repository's project path. */
const repository = process.env.GITHUB_REPOSITORY?.split("/").at(-1);
const base = process.env.PAGES_BASE || (repository ? `/${repository}/` : "/blacksite/");

export default defineConfig({
  base,
  server: { watch: { ignored: ["**/.blacksite/**"] } },
  plugins: [tailwindcss(), react()],
  resolve: { tsconfigPaths: true },
  build: {
    outDir: "dist-pages",
    emptyOutDir: true,
    // The game itself ships as one wasm module plus a thin shell, so the JS
    // budget is small. Keep this just under the real size so an accidental
    // static import of dev-only data (catalog JSON, Settings, the router)
    // fails the build loudly instead of silently bloating first load.
    chunkSizeWarningLimit: 300,
    rollupOptions: {
      input: "pages/index.html",
      output: {
        // Rolldown does not accept the object form of `manualChunks`; use
        // `codeSplitting.groups` so a dependency bump moves one cached chunk
        // instead of invalidating the whole bundle.
        codeSplitting: {
          groups: [
            { name: "react", test: /node_modules[\\/](react|react-dom|scheduler)[\\/]/ },
            { name: "radix", test: /node_modules[\\/]@radix-ui[\\/]/ },
            { name: "icons", test: /node_modules[\\/]lucide-react[\\/]/ },
          ],
        },
      },
    },
  },
});
