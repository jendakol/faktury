<template>
    <v-card width="1000" outlined rounded class="ma-0 pa-2 pt-1">
        <v-card-title>
            <h1 class="faktury-page-header">Current year overview</h1>
            <h2 class="faktury-sub-header" style="font-style: italic">(by creation date)</h2>
        </v-card-title>
        <YearOverviewChart ref="yearOverview" :year="new Date().getFullYear()" v-on:data-update="onDataUpdate"/>
    </v-card>
</template>

<script>
import YearOverviewChart from "../YearOverviewChart";

export default {
    name: 'DashboardYearOverview',
    components: {YearOverviewChart},
    methods: {
        onDataUpdate: function () {
            // let currentData = this.$refs.yearOverview.getCurrentData();
        },
        reloadData: function () {
            this.$refs.yearOverview.loadData()
        }
    }
}
</script>
