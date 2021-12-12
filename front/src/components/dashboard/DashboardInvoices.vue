<template>
    <v-card width="1000" outlined raised>
        <v-card-title>
            <h1 class="faktury-page-header">Your most recent invoices</h1>
        </v-card-title>
        <InvoicesTable :last='5' v-on:invoices-change="invoicesChange"/>
    </v-card>
</template>

<script>
import InvoicesTable from "../InvoicesTable";

export default {
    name: 'Invoices',
    components: {InvoicesTable},
    methods:{
        invoicesChange: function (){
            this.$emit('invoices-change')
        }
    }
}
</script>
