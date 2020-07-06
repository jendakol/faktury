<template>
    <v-card width="1200" outlined raised :loading="loading">
        <v-card-title>
            <v-text-field class="faktury-page-header" prefix="Invoice" solo v-model="invoiceData.code" counter="200"/>
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
                                                            input-class="invoice-datetime" v-bind="attrs"
                                                            v-on="on"
                                                            width="100%"
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
                                <ContactBox :code="entrepreneurData.code" :name="entrepreneurData.name"
                                            :address="entrepreneurData.address"/>
                            </v-col>
                            <v-spacer/>
                            <v-tooltip bottom>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-col cols="4" @click="$refs.ContactsDialog.show()" class="invoice-contact-box" v-bind="attrs"
                                           v-on="on">
                                        <ContactsDialog @contact-selected="contactSelected" ref="ContactsDialog"/>
                                        <ContactBox :code="contactData.code" :name="contactData.name" :address="contactData.address"/>
                                    </v-col>
                                </template>
                                <span>Click to change</span>
                            </v-tooltip>
                        </v-row>
                    </v-card-text>
                    <v-card-actions>
                        <v-spacer/>
                        <v-btn color="green darken-1" text @click="saveMetadata">Save</v-btn>
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

    export default {
        name: 'InvoiceDetail',
        components: {
            ContactsDialog,
            ContactBox,
            InvoiceRows,
            Datetime
        },
        mounted() {
            this.ajax("get/invoice-with-rows/" + this.$route.params.id, {}).then(invoice => {
                Promise.all([
                    this.ajax("get/contact/" + invoice.invoice.contactId, {}),
                    this.ajax("get/entrepreneur/" + invoice.invoice.entrepreneurId, {}),
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
            },
            saveMetadata: function () {
                console.log("Saving invoice metadata")
                console.log(this.invoiceData)

                this.asyncActionWithNotification("update/invoice", this.invoiceData, "Saving", (resp) => new Promise((success, error) => {
                        if (resp.success) {
                            success("Invoice saved")
                        } else {
                            error("Could not save invoice")
                        }
                    })
                );
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
        }
    }
</script>
