<template>
    <div>
        <v-card width="1000" outlined raised :loading="loading">
            <v-card-title>
                Your invoices
            </v-card-title>
            <v-card v-for="row in rows" :key="row.code" :to="{ name: 'InvoiceDetail', params: { id: row.id } }" hover link rounded>
                <v-card-text>
                    {{row.created}}
                </v-card-text>
                <v-card-actions>
                </v-card-actions>
            </v-card>
        </v-card>
    </div>
</template>

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
        methods: {}
    }
</script>
