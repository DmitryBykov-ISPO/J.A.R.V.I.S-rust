<script lang="ts">
    import { onMount, onDestroy } from "svelte"
    import { invoke } from "@tauri-apps/api/core"
    import { Notification, Space, Button } from "@svelteuidev/core"
    import { InfoCircled } from "radix-icons-svelte"

    import SearchBar from "@/components/elements/SearchBar.svelte"
    import ArcReactor from "@/components/elements/ArcReactor.svelte"
    import HDivider from "@/components/elements/HDivider.svelte"
    import Stats from "@/components/elements/Stats.svelte"
    import Footer from "@/components/Footer.svelte"

    import { isJarvisRunning, updateJarvisStats } from "@/stores"

    let running = false
    let launching = false

    isJarvisRunning.subscribe(value => {
        running = value
    })

    onMount(() => {
        document.body.classList.add("assist-page")
    })

    onDestroy(() => {
        document.body.classList.remove("assist-page")
    })

    async function runAssistant() {
        launching = true
        try {
            await invoke("run_jarvis_app")
            // wait a bit then check if it's running
            setTimeout(() => {
                updateJarvisStats()
                launching = false
            }, 2000)
        } catch (err) {
            console.error("Failed to run jarvis-app:", err)
            launching = false
        }
    }
</script>

<HDivider />

{#if !running}
    <Notification
        title="Внимание!"
        icon={InfoCircled}
        color="cyan"
        withCloseButton={false}
    >
        В данный момент ассистент не запущен.<br />
        Но вы всё еще можете изменять его настройки.<br />
        <br />

        <Button
            color="lime"
            radius="md"
            size="sm"
            uppercase
            ripple
            fullSize
            on:click={runAssistant}
            disabled={launching}
        >
            {launching ? "Запуск..." : "Запустить"}
        </Button>
    </Notification>
{:else}
    <ArcReactor />
{/if}

<HDivider noMargin />
<Stats />
<Footer />