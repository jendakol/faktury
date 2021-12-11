<template>
    <div>
        <DashboardInvoices class="mt-4"/>
        <DashboardContacts class="mt-4"/>
    </div>
</template>

<script>
import DashboardInvoices from "@/components/dashboard/DashboardInvoices";
import DashboardContacts from "@/components/dashboard/DashboardContacts";

export default {
    name: 'Dashboard',
    components: {
        DashboardContacts,
        DashboardInvoices
    },
    data() {
        return {}
    },
    methods: {}
}
</script>
