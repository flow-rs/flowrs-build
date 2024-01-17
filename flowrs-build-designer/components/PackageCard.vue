<template>
    <v-card class="mx-auto" max-width="368">
        <v-card-item :title="name.split('::').pop()">
            <template v-slot:subtitle>
                <v-icon icon="mdi-information-box-outline" size="18" class="me-1 pb-1"></v-icon>
                {{ name }}

            </template>
        </v-card-item>

        <v-card-text class="py-0">
            <v-divider class="mb-2"></v-divider>
            <v-row align="center" no-gutters>

                <v-col cols="12" v-if="value.inputs == null && value.outputs == null">
                    <v-icon icon="mdi-tag-outline" size="18" class="me-1 pb-1"></v-icon>
                    Type
                </v-col>
                <v-col cols="12" v-if="value.inputs != null || value.outputs != null">
                    <v-icon icon="mdi-tag-outline" size="18" class="me-1 pb-1"></v-icon>
                    Node
                </v-col>
            </v-row>
            <v-row align="center" no-gutters>


                <v-col class="text-h2" cols="6">
                    <v-list>

                        <v-list-item v-for="i in value.inputs" :key="i" :value="i" color="primary" rounded="xl" disabled>

                            <v-list-item-title><v-chip class="ma-2" color="green" label text-color="white">
                                    <v-icon start icon="mdi-arrow-right-bold-outline"></v-icon>
                                    input
                                </v-chip></v-list-item-title>
                        </v-list-item>
                    </v-list>
                </v-col>

                <v-col cols="6" class="text-right">
                    <v-list>

                        <v-list-item v-for="i in value.outputs" :key="i" :value="i" color="primary" rounded="xl" disabled>

                            <v-list-item-title><v-chip class="ma-2" color="red" label text-color="white">
                                    output
                                    <v-icon class="mx-1" start icon="mdi-arrow-right-bold-outline"></v-icon>
                                </v-chip></v-list-item-title>
                        </v-list-item>
                    </v-list>
                </v-col>
            </v-row>
            <v-divider class="mb-2" v-if="value.type_parameters!==null"></v-divider>
            <v-row align="center" no-gutters>
                <v-col cols="12" class="text-center">

                    <v-chip v-for="i in value.type_parameters" class="ma-2" color="blue" label text-color="white">
                        <v-icon start icon="mdi-alpha-t-box-outline"></v-icon>
                        {{ i }}

                    </v-chip>
                </v-col>
            </v-row>
        </v-card-text>
    </v-card>
</template>
  
<script>
export default {
    props: {
        name: String,
        value: Object,
    }


}
</script>