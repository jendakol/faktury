<template>
    <v-card outlined raised class="pa-3 dialog-contact-box" height="210">
        {{ name }}<br/>
        <div style="white-space: pre-line;">{{ address }}</div>
        <br/>
        {{ codeFormatted }}<br/>
        {{ vatFormatted }}
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
        vat: [String, Object]
    },
    computed: {
        codeFormatted: function () {
            if (this.code === undefined) return ""

            return this.code === "" ? "" : "IČO " + this.code // TODO hard coded lang value
        },
        vatFormatted: function () {
            return this.formatVat(this.vat)
        },
    }
}
</script>
