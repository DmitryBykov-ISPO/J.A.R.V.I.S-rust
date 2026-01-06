<script lang="ts">
    import { onMount, onDestroy } from "svelte"
    import { Router } from "@roxi/routify"
    import routes from "../.routify/routes.default.js"
    import { SvelteUIProvider } from "@svelteuidev/core"
    import Events from "./Events.svelte"

    import { 
        loadVoiceSetting, 
        loadAppInfo, 
        startStatsPolling, 
        stopStatsPolling 
    } from "@/stores"

    onMount(() => {
        loadVoiceSetting()
        loadAppInfo()
        startStatsPolling(5000)
    })

    onDestroy(() => {
        stopStatsPolling()
    })
</script>

<SvelteUIProvider themeObserver="dark" withNormalizeCSS withGlobalStyles>
    <Router {routes} />
</SvelteUIProvider>

<Events />
