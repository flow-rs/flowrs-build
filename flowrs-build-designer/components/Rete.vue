<template>
  <v-row align="center"
      justify="center">
    <v-col cols="12" sm="12" md="6" >
      <v-toolbar  dark="true">
        <v-btn to="/" nuxt>Project Select</v-btn>
        <v-btn to="/packageselect" nuxt>Package Select</v-btn>
        <v-btn to="compilerunandmetrics" nuxt>Compile, Run and Metrics</v-btn>
<!--        <v-btn @click="openNewTab">Compile And Run</v-btn>-->
        <v-btn>
          <v-icon>mdi-content-save-all-outline</v-icon> Save
        </v-btn>
      </v-toolbar>
    </v-col>
  </v-row>

  <div class="rete" ref="rete"></div>

</template>

<script lang="ts">
import { createEditor } from "../rete";

export default {
  async mounted() {
    await createEditor(this.$refs.rete);
  },
  methods: {
    openNewTab() {
      const url = "/compileandrun";
      window.open(url, "_blank");
    }
  }
};
</script>

<style scoped>
.rete {
  position: relative;
  width: 80vw;
  height: 90vh;
  font-size: 1rem;
  background: white;
  margin: 1em auto 3em auto;
  border-radius: 1em;
  text-align: left;
  border: 3px solid #55b881;
}
</style>
