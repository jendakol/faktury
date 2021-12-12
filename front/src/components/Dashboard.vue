<template>
    <div>
        <DashboardInvoices class="mt-4" v-on:invoices-change="reloadStats()"/>
        <DashboardContacts class="mt-4"/>
        <DashboardYearOverview ref="dashboardYearOverview" class="mt-4 mb-4"/>
    </div>
</template>

<script>
import DashboardInvoices from "@/components/dashboard/DashboardInvoices";
import DashboardContacts from "@/components/dashboard/DashboardContacts";
import DashboardYearOverview from "@/components/dashboard/DashboardYearOverview";

export default {
    name: 'Dashboard',
    components: {
        DashboardContacts,
        DashboardInvoices,
        DashboardYearOverview
    },
    data() {
        return {}
    },
    methods: {
        reloadStats: function () {
            this.$refs.dashboardYearOverview.reloadData()
        }
    }
}
</script>
