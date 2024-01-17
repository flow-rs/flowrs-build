<script setup lang="ts">

// The Component is used to display a short instruction about the control, run and metrics page. The placeholder
// is shown until a process is started. It gets replaced by the metrics of the running process.


</script>

<template>
  <v-row>
    <v-col>
      <v-card title="Compile, run and show metrics of your flow project"></v-card>
    </v-col>
  </v-row>
  <v-row class="dotted-red-line">
    <v-col col="3">
      <v-img src="~assets/images/help-1.png" aspect-ratio="2.5"></v-img>
    </v-col>
    <v-col col="3">
      <v-img src="~assets/images/help-2.png" aspect-ratio="2.9"></v-img>
    </v-col>
    <v-col col="6">
      <v-img src="~assets/images/help-3.png" aspect-ratio="2.5"></v-img>
    </v-col>
  </v-row>
</template>

<style scoped lang="scss">
.dotted-red-line {
  border: 3px dotted darkblue;
  margin-left: 40px;
  margin-right: 40px;
  border-radius: 20px;
}
</style>
