<template>
    <v-container>
        <InvoiceRow v-for="row in rows" :key="row.id" :invoiceId="invoiceId" :data="row"
                    @row-updated="rowUpdated"
                    @row-deleted="rowDeleted"
        />

        <v-divider class="pt-3"/>

        <InvoiceNewRow :invoiceId="invoiceId" @row-inserted="rowInserted" ref="newRowComponent"/>
    </v-container>

</template>

<script>
    import InvoiceRow from "./InvoiceRow";
    import InvoiceNewRow from "./InvoiceNewRow"

    export default {
        name: 'InvoiceRows',
        components: {
            InvoiceRow,
            InvoiceNewRow,
        },
        props: {
            invoiceId: Number,
            rows: Array
        },
        methods: {
            rowUpdated: function (row) {
                this.$emit('row-updated', row)
            },
            rowDeleted: function (id) {
                this.$emit('row-deleted', id)
            },
            rowInserted: function (row) {
                this.$refs.newRowComponent.reset()
                this.$emit('row-inserted', row)
            }
        }
    }
</script>
