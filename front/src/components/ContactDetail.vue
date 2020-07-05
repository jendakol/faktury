<template>
        <v-card width="1200" outlined raised :loading="loading">
            <v-card-title>
                Contact {{ contactData.code }}
            </v-card-title>

            <v-card-text>
                TODO
            </v-card-text>
        </v-card>

</template>

<script>
    export default {
        name: 'ContactDetail',
        components: {
        },
        mounted() {
            this.ajax("get/contact/" + this.$route.params.id, {}, 1000).then(r => {
                this.contactData = r;
                this.loading = false
            })
        },
        data() {
            return {
                loading: true,
                contactData: {},
            }
        },
        methods: {
        }
    }
</script>
