// https://nuxt.com/docs/api/configuration/nuxt-config
import vuetify, { transformAssetUrls } from 'vite-plugin-vuetify'

export default defineNuxtConfig({
  devtools: { enabled: true },
  build: {
    transpile: ['vuetify'],
  },
  modules: [
    (_options, nuxt) => {
      nuxt.hooks.hook('vite:extendConfig', (config) => {
        // @ts-expect-error
        config.plugins.push(vuetify({ autoImport: true }))
      })
    },
    //...
  ],
  vite: {
    vue: {
      template: {
        transformAssetUrls,
      },
    },
  },
})

// import Vue from '@vitejs/plugin-vue';
// export default defineNuxtConfig({
//     devtools: { enabled: true },
//     // buildModules: ['@nuxt/typescript-build'],
//     vite: {
//         server: {
//             proxy: {
//                 '/api': {
//                     target: 'http://localhost:3000', // Zieladresse Ihrer Rust-API
//                     changeOrigin: true, // Aktiviere die Änderung des Ursprungs, um CORS-Probleme zu vermeiden
//                     secure: false,
//                     ws: true,
//                 },
//             },
//         },
//     },
// })
