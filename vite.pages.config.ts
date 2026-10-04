import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

/** Explicit base wins; Pages workflows inherit their repository's project path. */
const repository = process.env.GITHUB_REPOSITORY?.split("/").at(-1);
const base = process.env.PAGES_BASE || (repository ? `/${repository}/` : "/blacksite/");

export default defineConfig({
  base,
  plugins: [tailwindcss(), react()],
  resolve: { tsconfigPaths: true },
  build: {
    outDir: "dist-pages",
    emptyOutDir: true,
    rollupOptions: {
      input: "pages/index.html",
    },
  },
});
