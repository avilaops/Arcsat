import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  root: path.resolve(__dirname, "src/embed"),
  base: "",
  build: {
    outDir: path.resolve(__dirname, "dist/embed/runtime"),
    emptyOutDir: true,
    manifest: false,
    lib: {
      name: "AvilaERPWidget",
      entry: path.resolve(__dirname, "src/embed/login-widget.tsx"),
      formats: ["iife"],
      fileName: () => "login-widget.js"
    }
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src")
    }
  }
});
