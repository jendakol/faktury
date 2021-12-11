<template>
    <div class="ma-0 pa-2 pt-1">
        <v-card v-for="invoice in invoices" :key="invoice.code"
                :to="{ name: 'InvoiceDetail', params: { id: invoice.id } }"
                class="ma-0 mt-1 pa-0"
                no-gutters
                hover
                link
                outlined
                rounded>
            <v-card-text>
                <v-row class="invoice-row d-flex pa-1">
                    <v-col class="align-center ma-0 pa-0 pt-1 col-code">{{ invoice.code }}</v-col>
                    <v-col class="align-center ma-0 pa-0 pt-1 col-created">{{ formatDate(invoice.created) }}</v-col>
                    <v-col class="align-center ma-0 pa-0 pt-1 col-contact">{{ invoice.contactName }}</v-col>
                    <v-col class="align-center ma-0 pa-0 pt-1 col-price">
                        {{ Number((invoice.priceSum).toFixed(2)) }} Kč
                        <v-tooltip top v-if="invoice.payed != null">
                            <template v-slot:activator="{ on, attrs }">
                                <v-btn x-small class="mb-1" icon v-bind="attrs" v-on="on" @click.stop.prevent="deleteInvoice(invoice.id)">
                                    <v-icon color="green lighten-1">mdi-check-circle</v-icon>
                                </v-btn>
                            </template>
                            <span>Payed {{ formatDate(invoice.payed) }}</span>
                        </v-tooltip>
                        <v-tooltip top v-else-if="isDelayedWithPayment(invoice)">
                            <template v-slot:activator="{ on, attrs }">
                                <v-btn x-small class="mb-1" icon v-bind="attrs" v-on="on" @click.stop.prevent="deleteInvoice(invoice.id)">
                                    <v-icon color="red lighten-1">mdi-exclamation-thick</v-icon>
                                </v-btn>
                            </template>
                            <span>Unpaid - delayed with payment!</span>
                        </v-tooltip>
                        <v-tooltip top v-else>
                            <template v-slot:activator="{ on, attrs }">
                                <v-btn x-small class="mb-1" icon v-bind="attrs" v-on="on" @click.stop.prevent="deleteInvoice(invoice.id)">
                                    <v-icon color="red lighten-1">mdi-close</v-icon>
                                </v-btn>
                            </template>
                            <span>Yet unpaid!</span>
                        </v-tooltip>
                    </v-col>
                    <v-col class="align-center ma-0 pa-0">
                        <v-container pa-0 ma-0 class="d-flex justify-end">
                            <v-tooltip top>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-btn small icon v-bind="attrs" v-on="on" @click.stop.prevent="copyInvoice(invoice.id)">
                                        <v-icon color="silver lighten-1">mdi-content-copy</v-icon>
                                    </v-btn>
                                </template>
                                <span>Copy</span>
                            </v-tooltip>
                            <v-tooltip top>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-btn v-if="invoice.payed == null" small icon v-bind="attrs" v-on="on" @click.stop.prevent="markAsPaid(invoice.id)">
                                        <v-icon color="green lighten-1">mdi-wallet-plus</v-icon>
                                    </v-btn>
                                    <v-btn small icon v-else>
                                        <v-icon color="#666">mdi-wallet-plus</v-icon>
                                    </v-btn>
                                </template>
                                <span>Mark as paid</span>
                            </v-tooltip>
                            <v-tooltip top>
                                <template v-slot:activator="{ on, attrs }">
                                    <v-btn small icon v-bind="attrs" v-on="on" @click.stop.prevent="deleteInvoice(invoice.id)">
                                        <v-icon color="red lighten-1">mdi-delete</v-icon>
                                    </v-btn>
                                </template>
                                <span>Delete</span>
                            </v-tooltip>
                        </v-container>
                    </v-col>
                </v-row>
            </v-card-text>
        </v-card>
    </div>
</template>

<style scoped lang="scss">
.invoice-row {
    font-size: large;
    color: white;
}

.col-code {
    font-weight: bold;
}

.col-created {

}

.col-contact a {
    color: white;
}

.col-price {
    text-align: right;
}
</style>

<script>
import {SnotifyPosition} from "vue-snotify";

// TODO filtering

export default {
    name: 'InvoicesTable',
    props: {
        last: Number
    },
    mounted() {
        let data = {}
        if (this.last !== undefined) {
            data["last"] = parseInt(this.last.toString())
        }
        this.ajax('data-get/invoices/' + this.getEntrepreneurId(), data).then(data => {
            this.invoices = data;
            this.loading = false;
        });
    },
    data() {
        return {
            loading: true,
            invoices: [],
            today: new Date()
        }
    },
    methods: {
        formatDate: function (isoString) {
            let date = new Date(isoString)
            return date.toLocaleDateString(this.$store.state.locale)
        },
        isDelayedWithPayment: function (invoice) {
            return new Date(invoice.payUntil).getTime() < this.today.getTime()
        },
        deleteInvoice: function (id) {
            this.$snotify.confirm('Really delete this invoice?', 'Delete', {
                timeout: 5000,
                closeOnClick: false,
                pauseOnHover: true,
                position: SnotifyPosition.centerCenter,
                buttons: [
                    {
                        text: 'Yes', action: (toast) => {
                            console.log("Deleting invoice " + id)

                            this.ajax("data-delete/invoice/" + id).then(r => {
                                if (r.success) {
                                    this.$set(this, 'invoices', this.lodash.filter(this.invoices, function (e) {
                                        return e.id !== id
                                    }))
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
        copyInvoice: function (id) {
            console.log("Copying invoice " + id)

            this.asyncActionWithNotification("data-copy/invoice/" + id, {}, "Saving", (resp) => new Promise((success, error) => {
                    if (resp.id >= 0) {
                        success("Invoice copied")
                        this.$router.push({name: 'InvoiceDetail', params: {id: resp.id}})
                    } else {
                        error("Could not save invoice")
                    }
                })
            );
        },
        markAsPaid: function (id) {
            console.log("Marking invoice " + id + " as paid")

            let i = this.lodash.findIndex(this.invoices, ['id', id])
            if (i >= 0) {
                const now = new Date()
                this.invoices[i].payed = now.toISOString().split('T')[0]
            } else {
                console.log("Couldn't find invoice with id " + id + ", weird!!")
                return
            }

            this.saveMetadata(id)
        },
        saveMetadata: function (id) {
            let invoiceData;
            let i = this.lodash.findIndex(this.invoices, ['id', id])
            if (i >= 0) {
                invoiceData = this.invoices[i]
            } else {
                console.log("Couldn't find invoice with id " + id + ", weird!!")
                return
            }

            if (this.loading) return;
            this.loading = true

            // fix date formats:
            invoiceData.created = invoiceData.created.substring(0, 10)
            invoiceData.payUntil = invoiceData.payUntil.substring(0, 10)
            invoiceData.payed = invoiceData.payed != null ? invoiceData.payed.substring(0, 10) : null


            console.log("Saving invoice metadata")
            console.log(invoiceData)

            this.ajax("data-update/invoice", invoiceData).then(r => {
                if (r.success) {
                    console.log("Saved!")
                    this.$set(this.invoices[i], invoiceData) // to trigger the change callbacks
                } else {
                    this.$snotify.error("Could not save the invoice!", "Saving")
                }

                this.loading = false
            }).catch(e => {
                this.showNotification("error", "Could not save the invoice!", "Saving")
                // this.$snotify.error("Could not save the invoice!", "Saving")
                console.log(e)
                this.loading = false
            })
        }
    }
}
</script>
