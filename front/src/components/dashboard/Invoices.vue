<template>
    <v-card width="1000" outlined raised>
        <v-card-title>
            <h1 class="faktury-page-header">Your recent invoices</h1>
        </v-card-title>
        <InvoicesTable :last='3'/>
    </v-card>
</template>

<script>
import InvoicesTable from "../InvoicesTable";

export default {
    name: 'Invoices',
    components: {InvoicesTable},
}
</script>
