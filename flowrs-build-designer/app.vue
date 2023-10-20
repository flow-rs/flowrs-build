<template>
  <h1>Packages list</h1>
  <div
      v-if="pending"
      class="spinner-wrapper"
  >
    <span class="loader"></span>
  </div>
  <div
      v-else
      class="product-wrapper"
  >

    <div
        v-for="fpac in flowrsPackages"
        class="card"
    >
      <div class="title">Package Name: {{ fpac.name }}</div>
      <div class="version">Package Version: {{ fpac.version }}</div>
      <div v-for="crate in fpac.crates">
        <div>Crates: {{ crate }}</div>
        <div>Crate Types: {{ crate.types }}</div>
        <div>Crate Modules: {{ crate.modules }}</div>

      </div>
<!--      <div class="thumbnail"><img :src="product.image"></div>-->
<!--      <div class="description">{{ product.description }}</div>-->
<!--      <div class="wrapper-meta">-->
<!--        <span class="price">${{ product.price }}</span>-->
<!--        <span class="rate">☆ {{ product.rating.rate }}</span>-->
<!--      </div>-->
    </div>
  </div>
</template>

<script setup lang="ts">
import Rete from './components/Rete.vue'

// const axios = useNuxtApp();
// const {
//   data: flowrspackages,
//     pending,
//     error
// } = await $api.packages.getPackages({server: false});
const { $api } = useNuxtApp();

const {
  data: flowrsPackages,
  pending,
  error
} = await $api.packages.getPackages();

console.log(flowrsPackages)

// const {data: p } = await useFetch('http://127.0.0.1:3000/packages/')

</script>

<style scoped>
.page :deep(.min-h-screen) {
  min-height: auto;
}

.page :deep(.py-14) {
  padding-top: 1.5rem;
  padding-bottom: 1.5rem;
}

.page :deep(.grid) {
  display: none;
}

.page :deep(footer) {
  display: none;
}
</style>
