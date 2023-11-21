<template>
  <v-autocomplete
      :label="data.typeName"
      :items="data.possibleValues"
      :item-title="getItemText"
      @pointerdown.stop=""
      @dblclick.stop=""
      @update:modelValue="data.onSelection"
  ></v-autocomplete>
</template>

<script lang="ts">
import {getCurrentInstance} from "vue";

import 'vuetify/styles'
import {createVuetify} from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import type {TypeDefinition} from "~/repository/modules/packages";

export default {
  props: ['data'],
  created() {
    // load vuetify --> https://github.com/retejs/rete/issues/656
    const ctx = getCurrentInstance()
    if (!ctx) {
      return
    }
    if (!ctx.appContext.app.hasVuetify) {
      ctx.appContext.app.hasVuetify = true;
      const vuetify = createVuetify({components, directives});
      ctx.appContext.app.use(vuetify);
    }
    console.log('Data', this.data)
  },
  methods: {
    getItemText(item: [string, TypeDefinition]): string {
      return `${item[0]}`;
    },
  }
}
</script>

<style>
.v-input__control {
background-color: white !important;
  border-radius: 15px;
}
</style>