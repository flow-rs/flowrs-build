<script setup lang="ts">


//TODO: extract compontens in own files

//TODO: add prefix for logs which project was executed/compiled etc.
</script>

<template>
  <v-container fluid>
    <v-row>
      <v-col>
        <v-card title="Metric Panel"></v-card>
      </v-col>
    </v-row>
    <v-row align-content="center">
      <v-col col="4">
        <v-card title="Metric XY">
          <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
        </v-card>
      </v-col>
      <v-col col="3">
        <v-card title="Metric XY">
          <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
        </v-card>
      </v-col>
      <v-col col="2">
        <v-card title="Metric XY">
          <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
        </v-card>
      </v-col>
      <v-col col="3">
        <v-card title="Metric XY">
          <v-img src="~assets/images/1340003.png" aspect-ratio="2.5"></v-img>
        </v-card>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="3">
        <ProjectActionPane></ProjectActionPane>
      </v-col>
      <v-col cols="9">
        <Terminal></Terminal>
      </v-col>
    </v-row>
  </v-container>
</template>

<style scoped>

</style>
