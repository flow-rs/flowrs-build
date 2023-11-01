import '@mdi/font/css/materialdesignicons.css'
import 'vuetify/styles'
import { createVuetify } from 'vuetify'

export default defineNuxtPlugin((app) => {
  const vuetify = createVuetify({
    defaults: {
      VBtn: {
        color: 'primary',
        rounded: true,
      },
    },
  })
  app.vueApp.use(vuetify)
})
