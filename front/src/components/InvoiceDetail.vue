<template>
    <v-card width="1200" outlined raised :loading="loading">
        <v-card-title>
            <v-text-field class="faktury-page-header" prefix="Invoice" solo v-model="invoiceData.code" type="number" counter="200"/>
        </v-card-title>
        <v-card-text>
            <v-container fluid>
                <v-card outline raised>
                    <v-card-text>
                        <v-row>
                            <v-spacer/>
                            <v-col cols="4">
                                <v-card outlined class="pl-3 pr-3">
                                    <v-row>
                                        <v-col cols="4">Created</v-col>
                                        <v-col cols="8">
                                            <v-tooltip right>
                                                <template v-slot:activator="{ on, attrs }">
                                                    <Datetime
                                                        v-model="invoiceData.created"
                                                        type="date"
                                                        value-zone="local"
                                                        input-class="invoice-datetime" v-bind="attrs"
                                                        v-on="on"
                                                    />
                                                </template>
                                                <span>Click to change</span>
                                            </v-tooltip>
                                        </v-col>
                                    </v-row>
                                </v-card>
                                <v-card outlined class="pl-3 pr-3 mt-1">
                                    <v-row>
                                        <v-col cols="4">Pay until</v-col>
                                        <v-col cols="8">
                                            <v-tooltip right>
                                                <template v-slot:activator="{ on, attrs }">
                                                    <Datetime
                                                        v-model="invoiceData.payUntil"
                                                        type="date"
                                                        value-zone="local"
                                                        input-class="invoice-datetime" v-bind="attrs"
                                                        v-on="on"
                                                    />
                                                </template>
                                                <span>Click to change</span>
                                            </v-tooltip>
                                        </v-col>
                                    </v-row>
                                </v-card>
                                <v-card outlined class="pl-3 pr-3 mt-1">
                                    <v-row>
                                        <v-col cols="4">Payed</v-col>
                                        <v-col cols="8">
                                            <v-tooltip right>
                                                <template v-slot:activator="{ on, attrs }">
                                                    <Datetime
                                                        v-model="invoiceData.payed"
                                                        type="date"
                                                        value-zone="local"
                                                        input-class="invoice-datetime" v-bind="attrs"
                                                        v-on="on"
                                                    />
                                                </template>
                                                <span>Click to change</span>
                                            </v-tooltip>
                                        </v-col>
                                    </v-row>
                                </v-card>
                            </v-col>
                        </v-row>
                        <v-row>
                            <v-col cols="4">
                                <ContactBox :code="entrepreneurData.code"
                                            :name="entrepreneurData.name"
                                            :address="entrepreneurData.address"
                                            :vat="entrepreneurData.vat"/>
                            </v-col>
                            <v-spacer/>
                            <v-tooltip bottom>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-col cols="4" @click="$refs.ContactsDialog.show()" class="invoice-contact-box" v-bind="attrs"
                                           v-on="on">
                                        <ContactBox :code="contactData.code"
                                                    :name="contactData.name"
                                                    :address="contactData.address"
                                                    :vat="contactData.vat"/>
                                        <ContactsDialog @contact-selected="contactSelected" ref="ContactsDialog"/>
                                    </v-col>
                                </template>
                                <span>Click to change</span>
                            </v-tooltip>
                        </v-row>
                    </v-card-text>
                    <v-card-actions>
                        <v-spacer/>
                        <v-btn color="orange darken-1" text @click="downloadInvoice">Download</v-btn>
                        <v-btn color="red darken-1" text @click="deleteInvoice">Delete</v-btn>
                    </v-card-actions>
                </v-card>
                <br/>
                <v-card outline raised>
                    <InvoiceRows :rows="invoiceRows" :invoiceId="Number($route.params.id)"
                                 @row-deleted="rowDeleted"
                                 @row-updated="rowUpdated"
                                 @row-inserted="rowInserted"/>
                </v-card>
            </v-container>
        </v-card-text>

        <form ref="DownloadForm" :action="downloadUrl" method="POST">
            <input type="hidden" name="auth" :value="$storage.get('login-session')">
        </form>
    </v-card>
</template>

<style scoped lang="scss">
.invoice-contact-box {
    cursor: pointer;
}

.invoice-datetime {
    color: white;
}

</style>

<script>
import ContactBox from "./ContactBox";
import InvoiceRows from "./invoice-detail/InvoiceRows";
import ContactsDialog from "./ContactsDialog";
import {Datetime} from 'vue-datetime'
import {SnotifyPosition} from "vue-snotify";

export default {
    name: 'InvoiceDetail',
    components: {
        ContactsDialog,
        ContactBox,
        InvoiceRows,
        Datetime
    },
    mounted() {
        this.ajax("data-get/invoice-with-rows/" + this.$route.params.id).then(invoice => {
            Promise.all([
                this.ajax("data-get/contact/" + invoice.invoice.contactId),
                this.ajax("data-get/entrepreneur/" + invoice.invoice.entrepreneurId),
            ]).then(([contact, entrepreneur]) => {
                this.invoiceData = invoice.invoice
                this.invoiceRows = invoice.rows
                this.contactData = contact
                this.entrepreneurData = entrepreneur

                this.loading = false
            })
        })
    },
    data() {
        return {
            loading: true,
            invoiceData: {},
            invoiceRows: [],
            contactData: {},
            entrepreneurData: {},
        }
    },
    methods: {
        contactSelected: function (contact) {
            this.contactData = contact
            this.invoiceData.contactId = contact.id
            this.saveMetadata()
        },
        saveMetadata: function () {
            if (this.loading) return;

            this.loading = true

            // fix date formats:
            this.invoiceData.created = this.invoiceData.created.substring(0, 10)
            this.invoiceData.payUntil = this.invoiceData.payUntil.substring(0, 10)
            this.invoiceData.payed = this.invoiceData.payed != null ? this.invoiceData.payed.substring(0, 10) : null

            console.log("Saving invoice metadata")
            console.log(this.invoiceData)


            this.ajax("data-update/invoice", this.invoiceData).then(r => {
                if (r.success) {
                    console.log("Saved!")
                } else {
                    this.$snotify.error("Could not save the invoice!", "Saving");
                }

                this.loading = false
            }).catch(e => {
                this.$snotify.error("Could not save the invoice!", "Saving");
                console.log(e)
                this.loading = false
            })
        },
        deleteInvoice: function () {
            this.$snotify.confirm('Really delete this whole invoice?', 'Delete', {
                timeout: 5000,
                closeOnClick: false,
                pauseOnHover: true,
                position: SnotifyPosition.centerCenter,
                buttons: [
                    {
                        text: 'Yes', action: (toast) => {
                            let id = this.$route.params.id

                            console.log("Deleting invoice row " + id)

                            this.ajax("data-delete/invoice/" + id, {}).then(r => {
                                if (r.success) {
                                    this.$router.replace("/invoices")
                                } else {
                                    this.$snotify.error("Could not delete the invoice!", "Delete");
                                }
                            })
                            this.$snotify.remove(toast.id);
                        }
                    },
                    {
                        text: 'No', action: (toast) => {
                            this.$snotify.remove(toast.id);
                        }
                    },
                ]
            });
        },
        downloadInvoice: function () {
            this.$refs.DownloadForm.submit()
        },
        rowUpdated: function (updatedRow) {
            let newRows = this.lodash.map(this.invoiceRows, function (row) {
                if (row.id === updatedRow.id) {
                    return updatedRow;
                } else {
                    return row;
                }
            })

            this.$set(this, 'invoiceRows', newRows)
        },
        rowDeleted: function (id) {
            this.$set(this, 'invoiceRows', this.lodash.filter(this.invoiceRows, function (e) {
                return e.id !== id
            }))
        },
        rowInserted: function (row) {
            console.log("Inserted new invoice row:" + JSON.stringify(row))

            this.$set(this, 'invoiceRows', this.lodash.concat(this.invoiceRows, row))
        }
    },
    computed: {
        downloadUrl: function () {
            return this.hostUrl + "/download/" + this.$route.params.id + "?auth=1"
        }
    },
    watch: {
        invoiceData: {
            handler(val, old) {
                if (!this.loading && old.created !== "") {
                    this.saveMetadata()
                }
            },
            deep: true
        }
    }
}
</script>
