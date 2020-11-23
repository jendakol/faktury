<template>
    <v-row justify="center">
        <v-dialog v-model="dialog.shown" max-width="500px" overlay-opacity="0.9">
            <v-card outlined raised>
                <v-card-title>
                    <span class="headline">Select contact</span>
                </v-card-title>
                <v-card-text>
                    <v-container fluid>
                        <v-autocomplete
                            v-model="model"
                            :items="contacts"
                            :loading="contactsLoading"
                            :search-input.sync="search"
                            color="white"
                            hide-no-data
                            hide-selected
                            item-text="name"
                            item-value="id"
                            placeholder="Type to search"
                            prepend-icon="mdi-database-search"
                            return-object/>
                    </v-container>
                </v-card-text>
            </v-card>
        </v-dialog>
    </v-row>
</template>


<script>
export default {
    name: 'ContactsDialog',
    data() {
        return {
            contactsLoading: false,
            model: null,
            search: null,
            contacts: [],
            dialog: {
                shown: false
            }
        }
    },
    mounted() {
        this.loadContacts()
    },
    methods: {
        show: function () {
            console.log("Showing contacts dialog")
            this.model = null
            this.dialog.shown = true
        },
        loadContacts() {
            // Items have already been loaded
            if (this.contacts.length > 0) return

            // Items have already been requested
            if (this.contactsLoading) return

            this.contactsLoading = true

            this.ajax("data-get/contacts/" + this.getEntrepreneurId(), {}).then(r => {
                this.contacts = r
            }).finally(() => (this.contactsLoading = false))
        }
    },
    watch: {
        search() {
            this.loadContacts()
        },
        model(value) {
            if (this.model != null) {
                this.dialog.shown = false
                console.log("Selected contact: ")
                console.log(value)
                this.$emit("contact-selected", value)
            }
        }
    },
}
</script>
