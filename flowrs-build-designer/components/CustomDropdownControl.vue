

<template>
  <v-autocomplete
      clearable
      v-model="model"
      :label="data.typeName"
      :items="data.possibleValues"
      @update:modelValue="data.onSelection"
  ></v-autocomplete>
</template>

<script lang="ts">
import {VAutocomplete} from "vuetify/components";
import { getCurrentInstance } from "vue";

import 'vuetify/styles'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'

export default {
  data () {
    return {
      model: null,
    }
  },
  props: ['data'],
  created() {
    // load vuetify --> https://github.com/retejs/rete/issues/656
    const ctx = getCurrentInstance()
    if (!ctx) {
      return
    }
    if(!ctx.appContext.app.hasVuetify){
      ctx.appContext.app.hasVuetify = true;
      const vuetify = createVuetify({components, directives });
      ctx.appContext.app.use(vuetify);
    }
    console.log('Data',this.data)
  },
  components: {
    VAutocomplete: VAutocomplete
  }
}
</script>