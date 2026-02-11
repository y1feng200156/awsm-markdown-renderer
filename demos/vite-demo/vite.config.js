import { defineConfig } from "vite";

export default defineConfig({
  server: {
    fs: {
      // Allow serving files from project root to access pkg/
      allow: ["../.."],
    },
  },
});
