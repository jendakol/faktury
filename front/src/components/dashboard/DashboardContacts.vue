<template>
    <v-card width="1000" outlined raised :loading="loading" class="ma-0 pa-2 pt-1">
        <v-card-title>
            <h1 class="faktury-page-header">Your most used contacts</h1>
        </v-card-title>
        <v-card v-for="contact in contacts" :key="contact.id"
                :to="{ name: 'ContactDetail', params: { id: contact.id } }"
                class="ma-0 mt-1 pa-0"
                no-gutters
                hover
                link
                outlined
                rounded>
            <v-card-text>
                <v-row class="contact-row d-flex pa-1">
                    <v-col cols="3" class="align-center ma-0 pa-0 pt-1 col-name">{{ contact.name }}</v-col>
                    <v-col class="align-center ma-0 pa-0 pt-1 col-addr">{{ contact.address }}</v-col>
                    <v-col class="align-center ma-0 pa-0 pt-1">
                        <v-container pa-0 ma-0 class="d-flex justify-end">
                            <v-tooltip top>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-btn small icon v-bind="attrs" v-on="on" @click.stop.prevent="newInvoice(contact.id)">
                                        <v-icon color="green lighten-1">mdi-text-box-plus</v-icon>
                                    </v-btn>
                                </template>
                                <span>Create new invoice for {{ contact.name }}</span>
                            </v-tooltip>
                        </v-container>
                    </v-col>
                </v-row>
            </v-card-text>
        </v-card>
    </v-card>
</template>

<style scoped lang="scss">
.contact-row {
    font-size: large;
    color: white;
}

.col-name {
    font-weight: bold;
}

</style>

<script>
export default {
    name: 'Contacts',
    mounted() {
        // TODO paremetrize number of months
        this.ajax('data-get/contacts/' + this.getEntrepreneurId(), {count: 3, lastMonths: 6}).then(data => {
            this.contacts = data;
            this.loading = false;
        });
    },
    data() {
        return {
            contacts: [],
            loading: true,
        }
    },
    methods: {
        newInvoice: function (contactId) {
            console.log("Adding new invoice for contact id " + contactId)

            let invoice = {
                accountId: this.getUserId(),
                entrepreneurId: this.getEntrepreneurId(),
                contactId: contactId
            };

            console.log("Adding new invoice: ")
            console.log(invoice)

            this.asyncActionWithNotification("data-insert/invoice", invoice, "Saving", (resp) => new Promise((success, error) => {
                    if (resp.id >= 0) {
                        success("Invoice created")
                        this.$router.push({name: 'InvoiceDetail', params: {id: resp.id}})
                    } else {
                        error("Could not save invoice")
                    }
                })
            );
        }
    }
}
</script>
