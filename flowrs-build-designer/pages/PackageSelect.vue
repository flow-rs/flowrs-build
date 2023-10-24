<template>
  <div>
    <Navbar />
    <PackageSelect />
  </div>
</template>

<script>
import PackageSelect from '../components/PackageSelect.vue'
import Navbar from '../components/Navbar.vue'

export default {
  components: {
    PackageSelect,
  },
}
</script>