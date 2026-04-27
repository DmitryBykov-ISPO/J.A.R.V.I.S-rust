<script lang="ts">
    import { Space } from "@svelteuidev/core"

    import HDivider from "@/components/elements/HDivider.svelte"
    import Footer from "@/components/Footer.svelte"
    import { translations, translate } from "@/stores"

    $: t = (key: string) => translate($translations, key)

    const builderUrl = "https://github.com/DmitryBykov-ISPO/J.A.R.V.I.S-py/tree/main/tools/command_builder"
</script>

<Space h="xl" />

<div class="commands-info">
    <h2>{t('commands-heading')}</h2>
    <p>{t('commands-desc')}</p>
    <a class="builder-link" href={builderUrl} target="_blank" rel="noopener noreferrer">
        {t('commands-builder-link')}
    </a>
</div>

<HDivider />
<Footer />

<style lang="scss">
    .commands-info {
        background: #0a1214;
        border: 1px solid rgba(82, 254, 254, 0.2);
        border-radius: 8px;
        padding: 1.25rem 1.5rem;
        color: #d6e2e6;

        h2 {
            margin: 0 0 0.75rem;
            font-size: 1.05rem;
            color: #52fefe;
            font-weight: 600;
            letter-spacing: 0.02em;
        }

        p {
            margin: 0 0 1rem;
            font-size: 0.85rem;
            line-height: 1.5;
            color: rgba(255, 255, 255, 0.65);
        }

        :global(code) {
            background: rgba(82, 254, 254, 0.08);
            color: #52fefe;
            padding: 0.05rem 0.35rem;
            border-radius: 3px;
            font-size: 0.8rem;
        }
    }

    .builder-link {
        display: inline-block;
        padding: 0.5rem 0.9rem;
        background: rgba(82, 254, 254, 0.08);
        border: 1px solid rgba(82, 254, 254, 0.4);
        border-radius: 6px;
        color: #52fefe;
        font-size: 0.8rem;
        text-decoration: none;
        transition: background 0.15s ease, border-color 0.15s ease;

        &:hover {
            background: rgba(82, 254, 254, 0.16);
            border-color: rgba(82, 254, 254, 0.6);
        }
    }
</style>
