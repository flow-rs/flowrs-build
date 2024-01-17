<script setup lang="ts">
import {useEventsStore} from "~/store/eventStore";
import {ContextCreator} from "~/rete/flowrs/contextCreator";
import {navigateTo} from "#app";
import {createEditor} from "~/rete";

const eventsStore = useEventsStore();

// subscribe to save button state on AppBar
eventsStore.$subscribe((mutation, state) => {
  if (state.isSaveButtonClicked) {
    handleSaveButtonClick();
  }
})

// executes saving process and manages alert banner content
const handleSaveButtonClick = async () => {
  eventsStore.setLoading(true)
  eventsStore.setErrorMessage("")
  eventsStore.setAlert(true)
  eventsStore.setSaveButtonClicked(false);
  ContextCreator.saveBuilderStateAsProject().then(() => {
    eventsStore.setLoading(false)
    eventsStore.setAlert(false)
    navigateTo("/");
  }).catch((e) => {
    console.error("Error caught", e);
    eventsStore.setLoading(false)
    eventsStore.setErrorMessage(e.message)
    eventsStore.setAlert(true)
  });

}
</script>

<script lang="ts">
import { defineComponent } from 'vue'
import { createEditor} from "~/rete";

export default defineComponent({
  mounted(){
    // init rete editor
    createEditor(this.$refs.rete)
  }
})
</script>

<template>
  <PackageDrawer/>
  <AlertComponent/>
  <div class="rete" ref="rete"></div>
</template>

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

.v-alert {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  width: auto; /* Adjust the width as needed */
  max-width: 95%;
  z-index: 9999; /* Ensure it's above other elements on the page */
}
</style>
