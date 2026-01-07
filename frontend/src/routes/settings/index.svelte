<script lang="ts">
    import { onMount } from "svelte"
    import { invoke } from "@tauri-apps/api/core"
    import { goto } from "@roxi/routify"
    import { setTimeout } from "worker-timers"

    import { showInExplorer } from "@/functions"
    import { appInfo, assistantVoice, translations, translate } from "@/stores"

    import HDivider from "@/components/elements/HDivider.svelte"
    import Footer from "@/components/Footer.svelte"

    import {
        Notification,
        Button,
        Text,
        Tabs,
        Space,
        Alert,
        Input,
        InputWrapper,
        NativeSelect,
        Switch
    } from "@svelteuidev/core"

    import {
        Check,
        Mix,
        Cube,
        Code,
        Gear,
        QuestionMarkCircled,
        CrossCircled
    } from "radix-icons-svelte"

    $: t = (key: string) => translate($translations, key)

    // ### STATE
    interface MicrophoneOption {
        label: string
        value: string
    }

    let availableMicrophones: MicrophoneOption[] = []
    let availableVoskModels: { label: string; value: string }[] = []
    let settingsSaved = false
    let saveButtonDisabled = false

    // form values (state vars)
    let voiceVal = ""
    let selectedMicrophone = ""
    let selectedWakeWordEngine = ""
    let selectedIntentRecognitionEngine = ""
    let selectedVoskModel = ""
    let selectedNoiseSuppression = ""
    let selectedVad = ""
    let gainNormalizerEnabled = false
    let apiKeyPicovoice = ""
    let apiKeyOpenai = ""

    // subscribe to stores
    assistantVoice.subscribe(value => {
        voiceVal = value
    })

    let feedbackLink = ""
    let logFilePath = ""
    appInfo.subscribe(info => {
        feedbackLink = info.feedbackLink
        logFilePath = info.logFilePath
    })

    // ### FUNCTIONS
    async function saveSettings() {
        saveButtonDisabled = true
        settingsSaved = false

        try {
            await Promise.all([
                invoke("db_write", { key: "assistant_voice", val: voiceVal }),
                invoke("db_write", { key: "selected_microphone", val: selectedMicrophone }),
                invoke("db_write", { key: "selected_wake_word_engine", val: selectedWakeWordEngine }),
                invoke("db_write", { key: "selected_intent_recognition_engine", val: selectedIntentRecognitionEngine }),
                invoke("db_write", { key: "selected_vosk_model", val: selectedVoskModel }),

                invoke("db_write", { key: "noise_suppression", val: selectedNoiseSuppression }),
                invoke("db_write", { key: "vad", val: selectedVad }),
                invoke("db_write", { key: "gain_normalizer", val: gainNormalizerEnabled.toString() }),

                invoke("db_write", { key: "api_key__picovoice", val: apiKeyPicovoice }),
                invoke("db_write", { key: "api_key__openai", val: apiKeyOpenai })
            ])

            // update shared store
            assistantVoice.set(voiceVal)
            settingsSaved = true

            // hide alert after 5 seconds
            setTimeout(() => {
                settingsSaved = false
            }, 5000)

            // restart listening with new settings
            // stopListening(() => startListening())
        } catch (err) {
            console.error("failed to save settings:", err)
        }

        setTimeout(() => {
            saveButtonDisabled = false
        }, 1000)
    }

    // ### INIT
    onMount(async () => {
        try {
            // load microphones
            const mics = await invoke<string[]>("pv_get_audio_devices")
            availableMicrophones = [
                { label: "По умолчанию (Система)", value: "-1" },  // system default
                ...mics.map((name, idx) => ({
                    label: name,
                    value: String(idx)
                }))
            ]

            // load vosk models
            const voskModels = await invoke<{ name: string; language: string; size: string }[]>("list_vosk_models")
            availableVoskModels = voskModels.map(m => ({
                label: `${m.name} (${m.language}, ${m.size})`,
                value: m.name
            }))

            // load settings from db
            const [mic, wakeWord, intentReco, voskModel,
                   noiseSuppression, vad, gainNormalizer,
                   pico, openai] = await Promise.all([
                invoke<string>("db_read", { key: "selected_microphone" }),
                invoke<string>("db_read", { key: "selected_wake_word_engine" }),
                invoke<string>("db_read", { key: "selected_intent_recognition_engine" }),
                invoke<string>("db_read", { key: "selected_vosk_model" }),

                invoke<string>("db_read", { key: "noise_suppression" }),
                invoke<string>("db_read", { key: "vad" }),
                invoke<string>("db_read", { key: "gain_normalizer" }),

                invoke<string>("db_read", { key: "api_key__picovoice" }),
                invoke<string>("db_read", { key: "api_key__openai" })
            ])

            selectedMicrophone = mic
            selectedWakeWordEngine = wakeWord
            selectedIntentRecognitionEngine = intentReco
            selectedVoskModel = voskModel
            selectedNoiseSuppression = noiseSuppression
            selectedVad = vad
            gainNormalizerEnabled = gainNormalizer === "true"
            apiKeyPicovoice = pico
            apiKeyOpenai = openai
        } catch (err) {
            console.error("failed to load settings:", err)
        }
    })
</script>

<Space h="xl" />

<Notification
    title={t('settings-beta-title')}
    icon={QuestionMarkCircled}
    color="blue"
    withCloseButton={false}
>
    {t('settings-beta-desc')}<br />
    {t('settings-beta-feedback')} <a href={feedbackLink} target="_blank">{t('settings-beta-bot')}</a>.
    <Space h="sm" />
    <Button
        color="gray"
        radius="md"
        size="xs"
        uppercase
        on:click={() => showInExplorer(logFilePath)}
    >
        {t('settings-open-logs')}
    </Button>
</Notification>

<Space h="xl" />

{#if settingsSaved}
    <Notification
        title={t('notification-saved')}
        icon={Check}
        color="teal"
        on:close={() => { settingsSaved = false }}
    />
    <Space h="xl" />
{/if}

<Tabs class="form" color="#8AC832" position="left">
    <Tabs.Tab label={t('settings-general')} icon={Gear}>
        <Space h="sm" />
        <NativeSelect
            data={[
                { label: "Jarvis New (ремастер)", value: "jarvis-remaster" },
                { label: "Рик из «Рик и Морти»", value: "rick-morty" },
                { label: "Jarvis (от Хауди)", value: "jarvis-howdy" },
                { label: "Jarvis OG (из фильмов)", value: "jarvis-og" }
            ]}
            label={t('settings-voice')}
            description={t('settings-voice-desc')}
            variant="filled"
            bind:value={voiceVal}
        />
    </Tabs.Tab>

    <Tabs.Tab label={t('settings-devices')} icon={Mix}>
        <Space h="sm" />
        <NativeSelect
            data={availableMicrophones}
            label={t('settings-microphone')}
            description={t('settings-microphone-desc')}
            variant="filled"
            bind:value={selectedMicrophone}
        />
    </Tabs.Tab>

    <Tabs.Tab label={t('settings-neural-networks')} icon={Cube}>
        <Space h="sm" />
        <NativeSelect
            data={[
                { label: "Rustpotter", value: "Rustpotter" },
                { label: "Vosk", value: "Vosk" },
                { label: "Picovoice Porcupine", value: "Picovoice" }
            ]}
            label={t('settings-wake-word-engine')}
            description={t('settings-wake-word-desc')}
            variant="filled"
            bind:value={selectedWakeWordEngine}
        />

        {#if selectedWakeWordEngine === "picovoice"}
            <Space h="sm" />
            <Alert title={t('settings-attention')} color="#868E96" variant="outline">
                <Notification
                    title={t('settings-picovoice-warning')}
                    icon={CrossCircled}
                    color="orange"
                    withCloseButton={false}
                >
                    {t('settings-picovoice-waiting')}
                </Notification>
                <Space h="sm" />
                <Text size="sm" color="gray">
                    {t('settings-picovoice-key-desc')}
                    <a href="https://console.picovoice.ai/" target="_blank">Picovoice Console</a>.
                </Text>
                <Space h="sm" />
                <Input
                    icon={Code}
                    placeholder={t('settings-picovoice-key')}
                    variant="filled"
                    autocomplete="off"
                    bind:value={apiKeyPicovoice}
                />
            </Alert>
        {/if}

        <Space h="xl" />
        {#key availableVoskModels}
        <NativeSelect
            data={[
                { label: t('settings-auto-detect'), value: "" },
                ...availableVoskModels
            ]}
            label={t('settings-vosk-model')}
            description={t('settings-vosk-model-desc')}
            variant="filled"
            bind:value={selectedVoskModel}
        />
        {/key}

        {#if availableVoskModels.length === 0}
            <Space h="sm" />
            <Alert title={t('settings-models-not-found')} color="orange" variant="outline">
                <Text size="sm" color="gray">
                    {t('settings-models-hint')}
                </Text>
            </Alert>
        {/if}

        <Space h="xl" />
        <NativeSelect
            data={[
                { label: "Intent Classifier", value: "IntentClassifier" },
                { label: "Rasa", value: "Rasa" }
            ]}
            label={t('settings-intent-engine')}
            description={t('settings-intent-engine-desc')}
            variant="filled"
            bind:value={selectedIntentRecognitionEngine}
        />

        <Space h="xl" />

        <NativeSelect
            data={[
                { label: t('settings-disabled'), value: "None" },
                { label: "Nnnoiseless", value: "Nnnoiseless" }
            ]}
            label={t('settings-noise-suppression')}
            description={t('settings-noise-suppression-desc')}
            variant="filled"
            bind:value={selectedNoiseSuppression}
        />

        <Space h="md" />

        <NativeSelect
            data={[
                { label: t('settings-disabled'), value: "None" },
                { label: "Energy", value: "Energy" },
                { label: "Nnnoiseless", value: "Nnnoiseless" }
            ]}
            label={t('settings-vad')}
            description={t('settings-vad-desc')}
            variant="filled"
            bind:value={selectedVad}
        />

        <Space h="md" />

        <InputWrapper label={t('settings-gain-normalizer')}>
            <Text size="sm" color="gray">
                {t('settings-gain-normalizer-desc')}
            </Text>
            <Space h="xs" />
            <Switch
                label={gainNormalizerEnabled ? t('settings-enabled') : t('settings-disabled')}
                bind:checked={gainNormalizerEnabled}
            />
        </InputWrapper>

        <Space h="xl" />

        <InputWrapper label={t('settings-openai-key')}>
            <Text size="sm" color="gray">
                {t('settings-openai-not-supported')}
            </Text>
            <Space h="sm" />
            <Input
                icon={Code}
                placeholder={t('settings-openai-key')}
                variant="filled"
                autocomplete="off"
                bind:value={apiKeyOpenai}
                disabled
            />
        </InputWrapper>
    </Tabs.Tab>
</Tabs>

<Space h="xl" />

<Button
    color="lime"
    radius="md"
    size="sm"
    uppercase
    ripple
    fullSize
    on:click={saveSettings}
    disabled={saveButtonDisabled}
>
    {t('settings-save')}
</Button>

<Space h="sm" />

<Button
    color="gray"
    radius="md"
    size="sm"
    uppercase
    fullSize
    on:click={() => $goto("/")}
>
    {t('settings-back')}
</Button>

<HDivider />
<Footer />
