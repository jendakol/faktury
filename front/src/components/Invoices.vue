<template>
    <v-card width="1000" outlined raised>
        <v-card-title>
            <h1 class="faktury-page-header">Your invoices</h1>
        </v-card-title>
        <InvoicesTable/>
    </v-card>
</template>

<script>
    import InvoicesTable from "./dashboard/InvoicesTable";

    export default {
        name: 'Invoices',
        components: {
            InvoicesTable
        },
        data() {
            return {}
        },
        methods: {}
    }
</script>
