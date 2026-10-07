import { defineConfig } from "vite";

export default defineConfig({
  base: "/rstats-cli/",
  build: {
    target: "es2022",
    outDir: "dist",
  },
});
