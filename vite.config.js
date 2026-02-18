import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { routify } from "@roxi/routify/vite-plugin";

export default defineConfig({
  plugins: [routify(), svelte()],
});
