<template>
    <div>
        404 NOT FOUND<br/>
        Go to
        <router-link :to="{name: 'Dashboard'}">dashboard!</router-link>
    </div>
</template>

<script>
export default {
    name: 'NotFound',
}
</script>
