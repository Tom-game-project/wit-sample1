import { defineConfig } from "vite"
import { viteSingleFile } from "vite-plugin-singlefile"
import { resolve } from "path"

export default defineConfig({
    // 1. ここでHTMLがあるディレクトリを指定します
    root: "dist", 

    plugins: [viteSingleFile()],

    build: {
        // rootを変更した場合、出力先(dist)が web/dist になってしまうのを防ぐため
        // プロジェクトルートの dist に出るように調整すると便利です
        outDir: resolve(__dirname, "dist"),
        emptyOutDir: true, // ビルド時にdistを空にする

        assetsInlineLimit: 100000000,
    },
})
