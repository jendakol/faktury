<template>
    <div>
        <InvoicesTable/>
    </div>
</template>

<script>
    import InvoicesTable from "./dashboard/InvoicesTable";

    export default {
        name: 'Dashboard',
        components: {
            InvoicesTable
        },
        data() {
            return {}
        },
        methods: {}
    }
</script>
