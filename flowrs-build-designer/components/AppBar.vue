<script setup lang="ts">
const route = useRoute()
const currentPage=route.name
console.log(currentPage)
</script>

<template>
  <v-app-bar >
    
    <v-img class="mx-2" src="~/assets/images/flowers-logo.png" max-height="40" max-width="40" contain></v-img>

    <v-toolbar-title>
      Flowrs - Flow Builder
    </v-toolbar-title>
    <v-btn v-if="currentPage=='index'" variant="elevated" class="ma-1" color="primary" to="/projectSelect">Save Project</v-btn>
    <v-btn  variant="elevated" class="ma-1" color="primary" to="/projectSelect">Project Select</v-btn>
    <v-btn  variant="elevated" class="ma-1" color="primary" to="/packageSelect">Package Select</v-btn>
    <v-btn  variant="elevated" class="ma-1" color="primary" to="/">Compile, Run and Metrics</v-btn>
    <v-spacer></v-spacer>
    <v-app-bar-nav-icon class="hidden-md-and-up" />
  </v-app-bar>
</template>

