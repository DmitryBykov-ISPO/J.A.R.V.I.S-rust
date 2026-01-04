<script lang="ts">
    import { onMount } from "svelte"
    import { invoke } from "@tauri-apps/api/core"
    import { goto } from "@roxi/routify"
    import { setTimeout } from "worker-timers"

    import { showInExplorer, stopListening, startListening } from "@/functions"
    import { appInfo, assistantVoice } from "@/stores"

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
        NativeSelect
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

    // ### STATE
    interface MicrophoneOption {
        label: string
        value: string
    }

    let availableMicrophones: MicrophoneOption[] = []
    let settingsSaved = false
    let saveButtonDisabled = false

    // form values
    let voiceVal = ""
    let selectedMicrophone = ""
    let selectedWakeWordEngine = ""
    let selectedIntentRecognitionEngine = ""
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
            stopListening(() => startListening())
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
            availableMicrophones = mics.map((name, idx) => ({
                label: name,
                value: String(idx)
            }))

            // load settings from db
            const [mic, wakeWord, intentReco, pico, openai] = await Promise.all([
                invoke<string>("db_read", { key: "selected_microphone" }),
                invoke<string>("db_read", { key: "selected_wake_word_engine" }),
                invoke<string>("db_read", { key: "selected_intent_recognition_engine" }),
                invoke<string>("db_read", { key: "api_key__picovoice" }),
                invoke<string>("db_read", { key: "api_key__openai" })
            ])

            selectedMicrophone = mic
            selectedWakeWordEngine = wakeWord
            selectedIntentRecognitionEngine = intentReco
            apiKeyPicovoice = pico
            apiKeyOpenai = openai
        } catch (err) {
            console.error("failed to load settings:", err)
        }
    })
</script>

<Space h="xl" />

<Notification
    title="БЕТА версия!"
    icon={QuestionMarkCircled}
    color="blue"
    withCloseButton={false}
>
    Часть функций может работать некорректно.<br />
    Сообщайте обо всех найденных багах в <a href={feedbackLink} target="_blank">наш телеграм бот</a>.
    <Space h="sm" />
    <Button
        color="gray"
        radius="md"
        size="xs"
        uppercase
        on:click={() => showInExplorer(logFilePath)}
    >
        Открыть папку с логами
    </Button>
</Notification>

<Space h="xl" />

{#if settingsSaved}
    <Notification
        title="Настройки сохранены!"
        icon={Check}
        color="teal"
        on:close={() => { settingsSaved = false }}
    />
    <Space h="xl" />
{/if}

<Tabs class="form" color="#8AC832" position="left">
    <!-- general tab -->
    <Tabs.Tab label="Общее" icon={Gear}>
        <Space h="sm" />
        <NativeSelect
            data={[
                { label: "Jarvis ремейк (от Хауди)", value: "jarvis-remake" },
                { label: "Jarvis OG (из фильмов)", value: "jarvis-og" }
            ]}
            label="Голос ассистента"
            description="Не все команды работают со всеми звуковыми пакетами."
            variant="filled"
            bind:value={voiceVal}
        />
    </Tabs.Tab>

    <!-- devices tab -->
    <Tabs.Tab label="Устройства" icon={Mix}>
        <Space h="sm" />
        <NativeSelect
            data={availableMicrophones}
            label="Выберите микрофон"
            description="Его будет слушать ассистент."
            variant="filled"
            bind:value={selectedMicrophone}
        />
    </Tabs.Tab>

    <!-- neural networks tab -->
    <Tabs.Tab label="Нейросети" icon={Cube}>
        <Space h="sm" />
        <NativeSelect
            data={[
                { label: "Rustpotter", value: "Rustpotter" },
                { label: "Vosk (медленный)", value: "Vosk" },
                { label: "Picovoice Porcupine (требует API ключ)", value: "Picovoice" }
            ]}
            label="Распознавание активационной фразы (Wake Word)"
            description="Выберите, какая нейросеть будет отвечать за распознавание активационной фразы."
            variant="filled"
            bind:value={selectedWakeWordEngine}
        />

        {#if selectedWakeWordEngine === "picovoice"}
            <Space h="sm" />
            <Alert title="Внимание!" color="#868E96" variant="outline">
                <Notification
                    title="Эта нейросеть работает не у всех!"
                    icon={CrossCircled}
                    color="orange"
                    withCloseButton={false}
                >
                    Мы ждем официального патча от разработчиков.
                </Notification>
                <Space h="sm" />
                <Text size="sm" color="gray">
                    Введите сюда свой ключ Picovoice.<br />
                    Он выдается бесплатно при регистрации в
                    <a href="https://console.picovoice.ai/" target="_blank">Picovoice Console</a>.
                </Text>
                <Space h="sm" />
                <Input
                    icon={Code}
                    placeholder="Ключ Picovoice"
                    variant="filled"
                    autocomplete="off"
                    bind:value={apiKeyPicovoice}
                />
            </Alert>
        {/if}

        <Space h="xl" />
        <NativeSelect
            data={[
                { label: "Intent Classifier", value: "IntentClassifier" },
                { label: "Rasa", value: "Rasa" }
            ]}
            label="Распознавание команд (Intent Recognition)"
            description="Выберите, какая нейросеть будет отвечать за распознавание команд."
            variant="filled"
            bind:value={selectedIntentRecognitionEngine}
        />

        <Space h="xl" />

        <InputWrapper label="Ключ OpenAI">
            <Text size="sm" color="gray">
                В данный момент ChatGPT <u>не поддерживается</u>.
                Он будет добавлен в ближайших обновлениях.
            </Text>
            <Space h="sm" />
            <Input
                icon={Code}
                placeholder="Ключ OpenAI"
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
    Сохранить
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
    Назад
</Button>

<HDivider />
<Footer />
