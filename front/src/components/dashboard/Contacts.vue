<template>
    <v-card width="1000" outlined raised :loading="loading">
        <v-card-title>
            <h1 class="faktury-page-header">Your most used contacts</h1>
        </v-card-title>
        <v-card flat
                v-for="contact in contacts" :key="contact.id">
            <v-row class="pl-3 pr-3 contact-row">
                <v-col cols="4" class="col-name">{{ contact.name }}</v-col>
                <v-col cols="7" class="col-addr">{{ contact.address }}</v-col>
                <v-col cols="1">
                    <v-tooltip top>
                        <template v-slot:activator="{ on, attrs }">
                            <v-btn icon v-bind="attrs" v-on="on" @click.stop.prevent="newInvoice(contact.id)">
                                <v-icon color="green lighten-1">mdi-text-box-plus</v-icon>
                            </v-btn>
                        </template>
                        <span>Create new invoice for {{ contact.name }}</span>
                    </v-tooltip>
                </v-col>
            </v-row>
        </v-card>
    </v-card>
</template>

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
