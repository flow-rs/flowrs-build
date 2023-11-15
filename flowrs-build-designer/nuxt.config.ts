// https://nuxt.com/docs/api/configuration/nuxt-config
import vuetify, {transformAssetUrls} from 'vite-plugin-vuetify'

export default defineNuxtConfig({
    css: ['~/assets/scss/main.scss'],
    devtools: {enabled: true},
    devServer: {
        port: 3001,
        host: '0.0.0.0'
    },
    imports: {
        dirs: ['stores'],
    },
    build: {
        transpile: ['vuetify'],
    },
    modules: [
        ['@pinia/nuxt',
            {
                autoImports: ['defineStore', 'acceptHMRUpdate'],
            },
        ],
        (_options, nuxt) => {
            nuxt.hooks.hook('vite:extendConfig', (config) => {
                // @ts-expect-error
                config.plugins.push(vuetify({autoImport: true}));
            });
        },
    ],
    vite: {
        vue: {
            template: {
                transformAssetUrls,
            },
        },
    },
});
