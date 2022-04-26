<template>
    <div>
        <BarChart :chartData="chartData" :options="options"/>
    </div>
</template>

<script>
import {BarChart} from 'vue-chart-3';

export default {
    name: 'YearOverviewChart',
    components: {BarChart},
    props: {
        year: Number,
    },
    data() {
        return {
            chartData: {
                labels: ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"],
                datasets: [
                    {
                        label: "Paid",
                        data: [],
                        backgroundColor: "lightgreen",
                    },
                    {
                        label: "Unpaid",
                        data: [],
                        backgroundColor: "orange",
                    },
                ],
            },
            options: {
                responsive: true,
                scales: {
                    x: {stacked: true},
                    y: {stacked: true},
                }
            }
        }
    },
    mounted() {
        this.loadData()
    },
    methods: {
        getCurrentData: function () {
            return this.chartData.datasets
        },
        loadData: function () {
            this.ajax("data-get/yearly-stats/" + this.year + "/" + this.getEntrepreneurId()).then((r) => {

                let paid = [];
                let unpaid = [];
                for (let i = 1; i <= 12; i++) {
                    if (r[i] === undefined) paid[i - 1] = 0; else paid[i - 1] = r[i].paid;
                    if (r[i] === undefined) unpaid[i - 1] = 0; else unpaid[i - 1] = r[i].unpaid;
                }
                // console.log(paid)
                // console.log(unpaid)
                this.chartData.datasets[0].data = paid
                this.chartData.datasets[1].data = unpaid

                this.$emit('data-update')
            })
        }
    }
}
</script>
