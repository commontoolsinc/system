import { defineConfig } from 'vite'
import { viteStaticCopy } from 'vite-plugin-static-copy';
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

const iconsPath = 'node_modules/@shoelace-style/shoelace/dist/assets/icons';

export default defineConfig({
    resolve: {
        alias: [
            {
                find: /\/assets\/icons\/(.+)/,
                replacement: `${iconsPath}/$1`,
            },
        ],
    },
    plugins: [
        wasm(),
        topLevelAwait(),
        viteStaticCopy({
            targets: [
                {
                    src: iconsPath,
                    dest: 'assets',
                },
            ],
        })
    ],
    server: {
        fs: {
            allow: [
                "./",
                "/nix/store"
            ]
        }
    }
})