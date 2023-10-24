<template>
  <div>
    <Navbar />
    <ProjectSelect />
  </div>
</template>

<script>
import ProjectSelect from '../components/ProjectSelect.vue'
import Navbar from '../components/Navbar.vue'

export default {
  components: {
    ProjectSelect,
  },
}
</script>