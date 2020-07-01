<template>
    <div>
        <InvoicesTable :ajax="this.ajax" :asyncActionWithNotification="this.asyncActionWithNotification"/>
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

<style scoped lang="scss">

    div#loginFormTabs {
        border: 1px solid black;
        border-radius: 5px;
        display: block;
        padding: 5px;
        width: 500px;
        height: 200px;
        position: absolute;
        left: 50%;
        top: 50%;
        margin: -100px 0 0 -250px;
    }

    .formInput {
        margin: 10px 0 0 0;
    }

    .formButton {
        margin: 10px 0 0 -75px;
        display: block;
        width: 150px;
        position: relative;
        left: 50%;
    }
</style>
