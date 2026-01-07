<script lang="ts">
    import { translations, translate } from "@/stores"
    
    $: t = (key: string) => translate($translations, key)
    
    let searchQuery = ""
</script>

<div id="search-form" class="search" class:active={searchQuery !== ""}>
    <form action="#" method="GET">
        <input
            bind:value={searchQuery}
            type="text"
            name="q"
            placeholder={t('search-placeholder')}
            autocomplete="off"
            minlength="3"
            maxlength="30"
        />
        <small>Enter</small>
    </form>
</div>