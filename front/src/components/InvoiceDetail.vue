<template>
    <div>
        INVOICE DETAIL {{ $route.params.id }}
        {{ data }}
    </div>
</template>

<script>
    export default {
        name: 'InvoiceDetail',
        mounted() {
            this.ajax("get/invoice/" + this.$route.params.id, {}, 1000).then(r=> {
                this.data = r;
            })
        },
        data() {
            return {
                data: {}
            }
        },
        methods: {}
    }
</script>
