<template>
        <v-card width="1200" outlined raised :loading="loading">
            <v-card-title>
                Invoice {{ invoiceData.code }}
            </v-card-title>

            <!--                {{ invoiceData }}-->

            <InvoiceRows :rows="invoiceRows" :invoiceId="$route.params.id"
                         @row-deleted="rowDeleted"
                         @row-updated="rowUpdated"
                         @row-inserted="rowInserted"/>
        </v-card>

</template>

<script>
    import InvoiceRows from "./invoice-detail/InvoiceRows";

    export default {
        name: 'InvoiceDetail',
        components: {
            InvoiceRows
        },
        mounted() {
            this.ajax("get/invoice-with-rows/" + this.$route.params.id, {}, 1000).then(r => {
                this.invoiceData = r.invoice;
                this.invoiceRows = r.rows;
                this.loading = false
            })
        },
        data() {
            return {
                loading: true,
                invoiceData: {},
                invoiceRows: [],
            }
        },
        methods: {
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
