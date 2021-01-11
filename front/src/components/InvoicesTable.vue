<template>
    <div>
        <v-card v-for="invoice in invoices" :key="invoice.code"
                :to="{ name: 'InvoiceDetail', params: { id: invoice.id } }"
                class="pa-0"
                hover
                link
                rounded>
            <v-card-text>
                <v-row class="invoice-row">
                    <v-col cols="2" class="col-code">{{ invoice.code }}</v-col>
                    <v-col cols="3" class="col-created">{{ formatDate(invoice.created) }}</v-col>
                    <v-col cols="3" class="col-contact">{{ invoice.contactName }}</v-col>
                    <v-spacer/>
                    <v-col cols="2" class="col-price">{{ Number((invoice.priceSum).toFixed(2)) }} Kč</v-col>
                    <v-col cols="1">
                        <v-tooltip top>
                            <template v-slot:activator="{ on, attrs }">
                                <v-btn icon v-bind="attrs" v-on="on" @click.stop.prevent="deleteInvoice(invoice.id)">
                                    <v-icon color="red lighten-1">mdi-delete</v-icon>
                                </v-btn>
                            </template>
                            <span>Delete invoice {{ invoice.code }}</span>
                        </v-tooltip>
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
        }
    },
    methods: {
        formatDate: function (isoString) {
            let date = new Date(isoString)
            return date.toLocaleDateString(this.$store.state.locale)
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
        }
    }
}
</script>
