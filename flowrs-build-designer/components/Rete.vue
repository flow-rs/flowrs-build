<template>
  <AlertComponent
  />
  <div class="rete" ref="rete"></div>

</template>

<script setup lang="ts">
import {useEventsStore} from "~/store/eventStore";

const userStore = useEventsStore()
const {isSaveButtonClicked} = storeToRefs(userStore)
</script>

<script lang="ts">
import {createEditor} from "~/rete";
import {useEventsStore} from "~/store/eventStore";
import {navigateTo} from "#app";
import {ContextCreator} from "~/rete/flowrs/contextCreator";

export default {
  mounted() {
    createEditor(this.$refs.rete).then(() => {
      console.log("Rete Editor loaded!")
    });
    const eventsStore = useEventsStore();

    eventsStore.$subscribe((mutation, state) => {
      if (state.isSaveButtonClicked) {
        this.handleSaveButtonClick();
      }
    })

  },
  data() {
    return {
      errorMessage: "",
      showAlert: false,
      isLoadingSave: false
    }
  },
  methods: {
    handleSaveButtonClick() {
      if (this.isLoadingSave) {
        return;
      }
      const eventsStore = useEventsStore();
      eventsStore.setLoading(true)
      eventsStore.setErrorMessage("")
      eventsStore.setAlert(true)
      eventsStore.setSaveButtonClicked(false);
      // Handle the save button click in the Rete component
      ContextCreator.saveBuilderStateAsProject().then(() => {
        useEventsStore().setLoading(false)
        useEventsStore().setAlert(false)
        navigateTo("/");
      }).catch((e) => {
        console.error("Error caught", e);
        useEventsStore().setLoading(false)
        useEventsStore().setErrorMessage(e.message)
        useEventsStore().setAlert(true)
      });

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

.v-alert {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  width: auto; /* Adjust the width as needed */
  max-width: 95%;
  z-index: 9999; /* Ensure it's above other elements on the page */
}
</style>
