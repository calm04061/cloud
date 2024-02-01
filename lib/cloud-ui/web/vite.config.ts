import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'
import {fileURLToPath, URL} from "node:url";
// https://vitejs.dev/config/
export default defineConfig({
    plugins: [vue()],
    resolve: {
        alias: {
            "~": fileURLToPath(new URL("./", import.meta.url)),
            "@": fileURLToPath(new URL("./src", import.meta.url)),
            "@data": fileURLToPath(new URL("./src/data", import.meta.url)),
        },
        extensions: [".js", ".json", ".jsx", ".mjs", ".ts", ".tsx", ".vue"],
    }
})
