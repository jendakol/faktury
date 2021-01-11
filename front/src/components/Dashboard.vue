<template>
    <div>
        <Invoices class="ma-5 mt-0"/>
        <Contacts class="ma-5"/>
    </div>
</template>

<script>
import Invoices from "@/components/dashboard/Invoices";
import Contacts from "@/components/dashboard/Contacts";

export default {
    name: 'Dashboard',
    components: {
        Contacts,
        Invoices
    },
    data() {
        return {}
    },
    methods: {}
}
</script>
