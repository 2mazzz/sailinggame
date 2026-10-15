import { resolve } from "node:path";
import { defineConfig } from "vite";

// Two pages: the game (index.html) and the training replay (training.html).
export default defineConfig({
  build: {
    rollupOptions: {
      input: { main: resolve(__dirname, "index.html"), training: resolve(__dirname, "training.html") },
    },
  },
});
