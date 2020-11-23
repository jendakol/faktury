<template>
    <v-card width="1000" outlined raised>
        <v-card-title>
            <h1 class="faktury-page-header">Your invoices</h1>
        </v-card-title>

        <InvoicesTable/>

        <v-divider/>
        <v-card-actions>
            <v-spacer/>
            <v-btn color="green darken-1" text @click="$refs.ContactsDialog.show()">New</v-btn>
        </v-card-actions>

        <ContactsDialog @contact-selected="saveNewInvoice" ref="ContactsDialog"/>
    </v-card>
</template>

<script>
import InvoicesTable from "./dashboard/InvoicesTable";
import ContactsDialog from "./ContactsDialog";

export default {
    name: 'Invoices',
    components: {
        ContactsDialog,
        InvoicesTable
    },
    data() {
        return {}
    },
    methods: {
        saveNewInvoice: function (contact) {
            console.log("Adding new invoice for contact id " + contact.id)

            let invoice = {
                accountId: this.getUserId(),
                entrepreneurId: this.getEntrepreneurId(),
                contactId: contact.id
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
        },
    }
}
</script>
