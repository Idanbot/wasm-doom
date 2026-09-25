import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

/** Served at https://idanbot.me/wasm-doom/ via the user-site domain. */
const base = process.env.PAGES_BASE || "/wasm-doom/";

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
