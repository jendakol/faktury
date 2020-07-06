<template>
    <v-card width="600" outlined raised :loading="loading">
        <v-card-title>
            <v-text-field class="faktury-page-header contact-name-text" prefix="Contact" solo v-model="contactData.name" counter="250"/>
        </v-card-title>

        <v-card-text>
            <v-row>
                <v-col>
                    <v-text-field label="Code" v-model="contactData.code" counter="100"/>
                </v-col>
            </v-row>
            <v-row>
                <v-col>
                    <v-textarea label="Address" v-model="contactData.address" counter="250"/>
                </v-col>
            </v-row>
        </v-card-text>
        <v-card-actions>
            <v-spacer/>
            <v-btn color="green darken-1" text @click="save">Save</v-btn>
        </v-card-actions>
    </v-card>
</template>

<style scoped lang="scss">
    .contact-name-text {
        /*font-size: 1.5em;*/
    }
</style>

<script>
    // TODO support enter-confirmation

    export default {
        name: 'ContactDetail',
        components: {},
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
            save: function () {
                console.log("Saving contact: ")
                console.log(this.contactData)

                this.asyncActionWithNotification("update/contact", this.contactData, "Saving", (resp) => new Promise((success, error) => {
                        if (resp.success) {
                            success("Contact saved")
                        } else {
                            error("Could not save contact")
                        }
                    })
                );
            }
        }
    }
</script>
