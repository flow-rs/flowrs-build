// https://nuxt.com/docs/api/configuration/nuxt-config
import Vue from '@vitejs/plugin-vue';
export default defineNuxtConfig({
  devtools: { enabled: true },
  // buildModules: ['@nuxt/typescript-build'],
  vite: {
    server: {
      proxy: {
        '/api': {
          target: 'http://localhost:3000', // Zieladresse Ihrer Rust-API
          changeOrigin: true, // Aktiviere die Änderung des Ursprungs, um CORS-Probleme zu vermeiden
          secure: false,
          ws: true,
        },
      },
    },
  },
})

// export default defineNuxtConfig({
//   nitro: {
//     devProxy: {
//       "/api": {
//         target:"your url",
//         changeOrigin: true,
//         prependPath: true,
//       }
//     }
//   },
// })
