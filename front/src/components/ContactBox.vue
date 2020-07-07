<template>
    <v-card outlined raised class="pa-3 dialog-contact-box" height="150">
        {{name}}<br/>
        {{code}}<br/>
        <div style="white-space: pre-line;">{{address}}</div>
    </v-card>
</template>

<style scoped lang="scss">
    .dialog-contact-box {
        color: white;
        font-size: 1.5em;
        line-height: 1.5em;
    }
</style>

<script>

    export default {
        name: 'ContactBox',
        props: {
            name: String,
            code: String,
            address: String,
        },
    }
</script>
