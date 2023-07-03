/// <reference types="vitest" />
// Plugins
import vue from "@vitejs/plugin-vue";
import vuetify from "vite-plugin-vuetify";

import AutoImport from "unplugin-auto-import/vite";

// Utilities
import {defineConfig} from "vite";
import {fileURLToPath, URL} from "node:url";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    // https://github.com/vuetifyjs/vuetify-loader/tree/next/packages/vite-plugin
    vuetify({
      autoImport: true,
      styles: {configFile: "src/styles/variables.scss"},
    }),
    AutoImport({
      imports: ["vue", "vue-router", "pinia"],
    }),
  ],
  define: {"process.env": {}},
  test: {
    globals: true,
    environment: "happy-dom",
  },
  resolve: {
    alias: {
      "~": fileURLToPath(new URL("./", import.meta.url)),
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@data": fileURLToPath(new URL("./src/data", import.meta.url)),
    },
    extensions: [".js", ".json", ".jsx", ".mjs", ".ts", ".tsx", ".vue"],
  },
  server: {
    port: 8080,
    host: '0.0.0.0',
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8088",
        changeOrigin: false,
      },
      "/dav": {
        target: "http://127.0.0.1:8088",
        changeOrigin: false,
      },
    }
  },
  css: {
    preprocessorOptions: {
      scss: {charset: false},
      css: {charset: false},
    },
  },
  cacheDir: ".vite_cache", // 将缓存目录设置为项目根目录下的 .vite_cache 文件夹
  build: {
    rollupOptions: {
      output: {
        manualChunks: function (id) {

          if (id.includes('node_modules')) {
            if (id.includes("echart")) {
              return "echart";
            }
            if (id.includes("microsoft-cognitiveservices-speech-sdk") || id.includes("openai")||id.includes("uuid")||id.includes("form-data")||id.includes("tslib")||id.includes("md-editor")||id.includes("clipboard")||id.includes("mitt")||id.includes("lottie-web")) {
              return "utils";
            }
            if (id.includes("axios")||id.includes("bent")) {
              return "net";
            }
            if (id.includes("vuetify")) {
              return "vuetify";
            }
            if (id.includes("vue")||id.includes("moment")||id.includes("devtools")||id.includes("pinia")||id.includes("intlify")) {
              return "vue";
            }
            // if (id.includes("faker")) {
            //   return "faker";
            // }
            if (id.includes("zrender")) {
              return "zrender";
            }
            // if (id.includes("devtools")||id.includes("pinia")||id.includes("intlify")) {
            //   return "devtools";
            // }
            console.log("vendor:" + id)
            return 'vendor';
          }
          // if (id.includes("vuetify")) {
          //   return "vuetify";
          // }
          // if (id.includes('components')||id.includes('landing')) {
          //   return "components";
          // }
          // if (id.includes('cloud')) {
          //   return "cloud";
          // }
          console.log("index:" + id)
          return "index";
        }
      }
    }
  },
});
