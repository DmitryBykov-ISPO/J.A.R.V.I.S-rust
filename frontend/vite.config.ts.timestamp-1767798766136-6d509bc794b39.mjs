// vite.config.ts
import { defineConfig } from "file:///D:/Rust/jarvis-app/frontend/node_modules/vite/dist/node/index.js";
import { svelte } from "file:///D:/Rust/jarvis-app/frontend/node_modules/@sveltejs/vite-plugin-svelte/src/index.js";
import sveltePreprocess from "file:///D:/Rust/jarvis-app/frontend/node_modules/svelte-preprocess/dist/index.js";
import tsconfigPaths from "file:///D:/Rust/jarvis-app/frontend/node_modules/vite-tsconfig-paths/dist/index.mjs";
import routify from "file:///D:/Rust/jarvis-app/frontend/node_modules/@roxi/routify/lib/extra/vite-plugin/vite-plugin.js";
var vite_config_default = defineConfig({
  plugins: [
    svelte({
      preprocess: [
        sveltePreprocess({
          typescript: true
        })
      ],
      onwarn: (warning, handler) => {
        const { code, frame } = warning;
        if (code === "css-unused-selector")
          return;
        handler(warning);
      }
    }),
    routify(),
    tsconfigPaths()
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCJEOlxcXFxSdXN0XFxcXGphcnZpcy1hcHBcXFxcZnJvbnRlbmRcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfZmlsZW5hbWUgPSBcIkQ6XFxcXFJ1c3RcXFxcamFydmlzLWFwcFxcXFxmcm9udGVuZFxcXFx2aXRlLmNvbmZpZy50c1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9pbXBvcnRfbWV0YV91cmwgPSBcImZpbGU6Ly8vRDovUnVzdC9qYXJ2aXMtYXBwL2Zyb250ZW5kL3ZpdGUuY29uZmlnLnRzXCI7aW1wb3J0IHsgZGVmaW5lQ29uZmlnIH0gZnJvbSBcInZpdGVcIjtcclxuaW1wb3J0IHsgc3ZlbHRlIH0gZnJvbSBcIkBzdmVsdGVqcy92aXRlLXBsdWdpbi1zdmVsdGVcIjtcclxuaW1wb3J0IHN2ZWx0ZVByZXByb2Nlc3MgZnJvbSBcInN2ZWx0ZS1wcmVwcm9jZXNzXCI7XHJcbmltcG9ydCB0c2NvbmZpZ1BhdGhzIGZyb20gJ3ZpdGUtdHNjb25maWctcGF0aHMnXHJcbmltcG9ydCByb3V0aWZ5IGZyb20gJ0Byb3hpL3JvdXRpZnkvdml0ZS1wbHVnaW4nXHJcblxyXG5leHBvcnQgZGVmYXVsdCBkZWZpbmVDb25maWcoe1xyXG4gIHBsdWdpbnM6IFtcclxuICAgIHN2ZWx0ZSh7XHJcbiAgICAgIHByZXByb2Nlc3M6IFtcclxuICAgICAgICBzdmVsdGVQcmVwcm9jZXNzKHtcclxuICAgICAgICAgIHR5cGVzY3JpcHQ6IHRydWUsXHJcbiAgICAgICAgfSksXHJcbiAgICAgIF0sXHJcbiAgICAgIG9ud2FybjogKHdhcm5pbmcsIGhhbmRsZXIpID0+IHtcclxuICAgICAgICBjb25zdCB7IGNvZGUsIGZyYW1lIH0gPSB3YXJuaW5nO1xyXG4gICAgICAgIGlmIChjb2RlID09PSBcImNzcy11bnVzZWQtc2VsZWN0b3JcIilcclxuICAgICAgICAgICAgcmV0dXJuO1xyXG5cclxuICAgICAgICBoYW5kbGVyKHdhcm5pbmcpO1xyXG4gICAgICB9LFxyXG4gICAgfSksXHJcbiAgICByb3V0aWZ5KCksXHJcbiAgICB0c2NvbmZpZ1BhdGhzKClcclxuICBdLFxyXG5cclxuICBjbGVhclNjcmVlbjogZmFsc2UsXHJcbiAgc2VydmVyOiB7XHJcbiAgICBwb3J0OiAxNDIwLFxyXG4gICAgc3RyaWN0UG9ydDogdHJ1ZSxcclxuICB9LFxyXG4gIGVudlByZWZpeDogW1wiVklURV9cIiwgXCJUQVVSSV9cIl0sXHJcbiAgYnVpbGQ6IHtcclxuICAgIHRhcmdldDogcHJvY2Vzcy5lbnYuVEFVUklfUExBVEZPUk0gPT0gXCJ3aW5kb3dzXCIgPyBcImNocm9tZTEwNVwiIDogXCJzYWZhcmkxM1wiLFxyXG4gICAgbWluaWZ5OiAhcHJvY2Vzcy5lbnYuVEFVUklfREVCVUcgPyBcImVzYnVpbGRcIiA6IGZhbHNlLFxyXG4gICAgc291cmNlbWFwOiAhIXByb2Nlc3MuZW52LlRBVVJJX0RFQlVHLFxyXG4gIH0sXHJcbn0pOyJdLAogICJtYXBwaW5ncyI6ICI7QUFBMlEsU0FBUyxvQkFBb0I7QUFDeFMsU0FBUyxjQUFjO0FBQ3ZCLE9BQU8sc0JBQXNCO0FBQzdCLE9BQU8sbUJBQW1CO0FBQzFCLE9BQU8sYUFBYTtBQUVwQixJQUFPLHNCQUFRLGFBQWE7QUFBQSxFQUMxQixTQUFTO0FBQUEsSUFDUCxPQUFPO0FBQUEsTUFDTCxZQUFZO0FBQUEsUUFDVixpQkFBaUI7QUFBQSxVQUNmLFlBQVk7QUFBQSxRQUNkLENBQUM7QUFBQSxNQUNIO0FBQUEsTUFDQSxRQUFRLENBQUMsU0FBUyxZQUFZO0FBQzVCLGNBQU0sRUFBRSxNQUFNLE1BQU0sSUFBSTtBQUN4QixZQUFJLFNBQVM7QUFDVDtBQUVKLGdCQUFRLE9BQU87QUFBQSxNQUNqQjtBQUFBLElBQ0YsQ0FBQztBQUFBLElBQ0QsUUFBUTtBQUFBLElBQ1IsY0FBYztBQUFBLEVBQ2hCO0FBQUEsRUFFQSxhQUFhO0FBQUEsRUFDYixRQUFRO0FBQUEsSUFDTixNQUFNO0FBQUEsSUFDTixZQUFZO0FBQUEsRUFDZDtBQUFBLEVBQ0EsV0FBVyxDQUFDLFNBQVMsUUFBUTtBQUFBLEVBQzdCLE9BQU87QUFBQSxJQUNMLFFBQVEsUUFBUSxJQUFJLGtCQUFrQixZQUFZLGNBQWM7QUFBQSxJQUNoRSxRQUFRLENBQUMsUUFBUSxJQUFJLGNBQWMsWUFBWTtBQUFBLElBQy9DLFdBQVcsQ0FBQyxDQUFDLFFBQVEsSUFBSTtBQUFBLEVBQzNCO0FBQ0YsQ0FBQzsiLAogICJuYW1lcyI6IFtdCn0K
