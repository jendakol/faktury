<template>
    <div>
        <v-card width="1000" outlined raised :loading="loading">
            <v-card-title>
                Your invoices
            </v-card-title>
            <v-card v-for="row in rows" :key="row.code"
                    :to="{ name: 'InvoiceDetail', params: { id: row.id } }"
                    class="pa-0"
                    hover
                    link
                    rounded>
                <v-card-text>
                    <v-row class="invoice-row">
                        <v-col cols="2" class="col-code">{{row.code}}</v-col>
                        <v-col cols="3" class="col-created">{{formatDate(row.created)}}</v-col>
                        <v-col cols="3" class="col-contact">
                            <router-link :to="{ name: 'ContactDetail', params: { id: row.contactId } }">
                                {{row.contactName}}
                            </router-link>
                        </v-col>
                        <v-spacer/>
                        <v-col cols="2" class="col-price">{{Number((row.priceSum).toFixed(2))}} Kč</v-col>
                    </v-row>
                </v-card-text>
            </v-card>
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
    export default {
        name: 'InvoicesTable',
        mounted() {
            this.ajax('get/invoices/1', {}, 1000).then(data => {
                this.rows = data;
                this.loading = false;
            });
        },
        data() {
            return {
                loading: true,
                rows: [],
            }
        },
        methods: {
            formatDate: function (isoString) {
                let date = new Date(isoString)
                return date.toLocaleString("cs-CZ")
            }
        }
    }
</script>
