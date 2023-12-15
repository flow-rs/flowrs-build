<template>
  <div class="rete" ref="rete"></div>

</template>

<script setup>
import {useEventsStore} from "~/store/eventStore";

const userStore = useEventsStore()
const {isSaveButtonClicked} = storeToRefs(userStore)
</script>

<script lang="ts">
import {createEditor} from "~/rete";
import {useEventsStore} from "~/store/eventStore";
import {ref} from "vue";
import {navigateTo} from "#app";

export default {
  mounted() {
    createEditor(this.$refs.rete).then(() => {
      console.log("Rete Editor loaded!")
    });

    useEventsStore.$subscribe((mutation,state) => {

      console.log("Save clicked!", mutation, state);
      navigateTo("/");
    })
  },
  methods: {
    handleSaveButtonClick() {
      console.log("Save clicked!");
      navigateTo("/");
      // Handle the save button click in the Rete component
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
