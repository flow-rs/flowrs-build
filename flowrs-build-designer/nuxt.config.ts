// https://nuxt.com/docs/api/configuration/nuxt-config
import vuetify, {transformAssetUrls} from 'vite-plugin-vuetify'

export default defineNuxtConfig({
    devtools: {enabled: true},
    buildModules: [
        '@nuxtjs/vuetfiy',
    ],
    vuetify: {
      theme: {
          defaultTheme: 'light',
          themes: {
              dark: {
                  primary: '#242f57'
              },
              light: {
                  primary: '#2face2',
                  secondary: '#242f57',
                  accent: '#30E3A3',
                  error: '#ff5722',
                  info: '#0099CC',
                  warning: '#ffbb33',
                  success: '#007E33'
              }
          }
      }
    },
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
