<script lang="ts">
    import { onMount, onDestroy } from "svelte"
    import { invoke } from "@tauri-apps/api/core"
    import { capitalizeFirstLetter } from "@/functions"

    import {
        Text,
    } from "@svelteuidev/core"

    let jarvisStats = { running: false, ram_mb: 0, cpu_usage: 0 }

    let microphoneLabel = ""
    let wakeWordEngine = ""
    let sttEngine = "Vosk"
    // let ramUsage = "-"

    let statsUpdateInterval: number | null = null

    async function updateStats() {
        try {
            jarvisStats = await invoke<{running: boolean, ram_mb: number, cpu_usage: number}>("get_jarvis_app_stats")
            //const usage = await invoke<number>("get_current_ram_usage")
            //ramUsage = usage.toFixed(2)
        } catch (err) {
            console.error("failed to get ram usage:", err)
        }
    }

    onMount(async () => {
        // start polling ram usage
        updateStats()
        statsUpdateInterval = setInterval(updateStats, 5000) as unknown as number

        try {
            // load microphone info
            const micIndex = Number(await invoke<string>("db_read", { key: "selected_microphone" }))
            microphoneLabel = await invoke<string>("pv_get_audio_device_name", { idx: micIndex })

            // load wake word engine
            const engine = await invoke<string>("db_read", { key: "selected_wake_word_engine" })
            wakeWordEngine = capitalizeFirstLetter(engine)
        } catch (err) {
            console.error("failed to load stats:", err)
        }
    })

    onDestroy(() => {
        if (statsUpdateInterval) {
            clearInterval(statsUpdateInterval)
        }
    })
</script>

<div class="statistics">
    <div class="online">
        <div class="pulse"><div class="wave"></div></div>
        <div class="info">
            <span class="num">Микрофон</span>
            <small title={microphoneLabel}>{microphoneLabel}</small>
        </div>
    </div>

    <div class="files">
        <div class="pulse"><div class="wave"></div></div>
        <div class="info">
            <span class="num">Нейросети</span>
            <small>{wakeWordEngine} + {sttEngine}</small>
        </div>
    </div>

    <div class="downloads hint--bottom" aria-label="Общее количество скачиваний по всему проекту">
        <div class="pulse"><div class="wave"></div></div>
        <div class="info">
            <span class="num">Ресурсы</span>
            {#if jarvisStats.running}
                <small>RAM: {jarvisStats.ram_mb} MB</small>
                <!--<Text>CPU: {jarvisStats.cpu_usage.toFixed(1)}%</Text>-->
            {:else}
                <Text color="gray">-</Text>
            {/if}
        </div>
    </div>
</div>

<style lang="scss">
    .statistics {
        position: relative;
        z-index: 3;
        padding: 0 10px;
        height: 100px;
        display: flex;
        justify-content: space-between;

        & > div {
            height: 70px;
        }

        .info {
            z-index: 10;
        }

        // [ Online/Microphone stat ]--
        & > .online {
            position: relative;
            width: 40%;

            $base-color: rgba(0, 191, 8, 1);
            $mid-color: rgba(0, 191, 8, 0.4);
            $end-color: rgba(0, 191, 8, 0);

            & > .pulse::before {
                background-color: $base-color;
            }

            & > .pulse::after {
                background-color: $base-color;
                animation: online-cdot linear 3s infinite forwards;
            }

            & > .pulse .wave {
                background-color: $mid-color;
                animation: online-radarWave cubic-bezier(0, 0.54, 0.53, 1) 3s 0s infinite;
            }

            & > .pulse .wave::after {
                background-color: $mid-color;
                animation: online-radarWave cubic-bezier(0, 0.54, 0.53, 1) 3s 0.1s infinite;
            }

            & > .info {
                position: absolute;
                top: 26px;
                left: 26px;

                & > span.num {
                    font-size: 18px;
                    font-weight: bold;
                    color: #00bf08;
                }

                & > small {
                    display: block;
                    color: #535a60;
                    font-size: 12px;
                    position: relative;
                    top: 0;
                    width: 130px;
                    max-height: 40px;
                    overflow: hidden;
                    line-height: 1.5em;
                }
            }

            @keyframes online-cdot {
                0% { opacity: 0.3; background: $base-color; }
                50% { opacity: 0.5; }
                100% { opacity: 1; background: $end-color; }
            }

            @keyframes online-radarWave {
                0% { opacity: 0.1; transform: scale(0); }
                5% { background: $mid-color; opacity: 1; }
                100% { transform: scale(1.2); background: $end-color; }
            }
        }

        // [ Files/Neural networks stat ]--
        & > .files {
            position: relative;
            width: 35%;

            $base-color: rgba(255, 129, 48, 1);
            $mid-color: rgba(255, 129, 48, 0.4);
            $end-color: rgba(255, 129, 48, 0);

            & > .pulse::before {
                background-color: $base-color;
            }

            & > .pulse::after {
                background-color: $base-color;
                animation: files-cdot linear 5s infinite forwards;
            }

            & > .pulse .wave {
                background-color: $mid-color;
                animation: files-radarWave cubic-bezier(0, 0.54, 0.53, 1) 5s 0s infinite;
            }

            & > .pulse .wave::after {
                background-color: $mid-color;
                animation: files-radarWave cubic-bezier(0, 0.54, 0.53, 1) 5s 0.1s infinite;
            }

            & > .info {
                position: absolute;
                top: 26px;
                left: 26px;

                & > span.num {
                    font-size: 18px;
                    font-weight: bold;
                    color: #ff8130;
                }

                & > small {
                    display: block;
                    color: #535a60;
                    font-size: 12px;
                    position: relative;
                    top: 0;
                }
            }

            @keyframes files-cdot {
                0% { opacity: 0.3; background: $base-color; }
                50% { opacity: 0.5; }
                100% { opacity: 1; background: $end-color; }
            }

            @keyframes files-radarWave {
                0% { opacity: 0.1; transform: scale(0); }
                5% { background: $mid-color; transform: scale(0.2); opacity: 1; }
                100% { transform: scale(0.8); background: $end-color; }
            }
        }

        // [ Downloads/Resources stat ]--
        & > .downloads {
            position: relative;

            $base-color: rgba(11, 66, 166, 1);
            $mid-color: rgba(32, 150, 243, 0.4);
            $end-color: rgba(32, 150, 243, 0);

            & > .pulse::before {
                background: rgba(32, 150, 243, 1);
            }

            & > .pulse::after {
                background: rgba(32, 150, 243, 1);
                animation: downloads-cdot linear 7s infinite forwards;
                animation-delay: 1s;
            }

            & > .pulse .wave {
                background-color: $mid-color;
                animation: downloads-radarWave cubic-bezier(0, 0.54, 0.53, 1) 7s 0s infinite;
                animation-delay: 1s;
            }

            & > .pulse .wave::after {
                background-color: $mid-color;
                animation: downloads-radarWave cubic-bezier(0, 0.54, 0.53, 1) 7s 0.1s infinite;
                animation-delay: 1s;
            }

            & > .info {
                position: absolute;
                top: 26px;
                left: 26px;

                & > span.num {
                    font-size: 18px;
                    font-weight: bold;
                    color: #1b78a6;
                }

                & > small {
                    display: block;
                    color: #535a60;
                    font-size: 12px;
                    position: relative;
                    top: 0;
                }
            }

            @keyframes downloads-cdot {
                0% { opacity: 0.3; background: $base-color; }
                50% { opacity: 0.5; }
                100% { opacity: 1; background: $end-color; }
            }

            @keyframes downloads-radarWave {
                0% { opacity: 0.1; transform: scale(0); }
                5% { background: $mid-color; opacity: 1; }
                100% { transform: scale(0.7); background: $end-color; }
            }
        }

        // [ Shared pulse styles ]--
        .pulse {
            position: relative;
            height: 100px;
            width: 100px;
            margin: 0;
            left: -43px;
            top: 0px;
            z-index: 5;
        }

        .pulse::before {
            content: "";
            position: absolute;
            width: 11px;
            height: 11px;
            border-radius: 50%;
            left: 50%;
            top: 50%;
            transform: translate(-50%, -50%);
            opacity: 0.5;
        }

        .pulse::after {
            content: "";
            position: absolute;
            width: 20px;
            height: 20px;
            border-radius: 50%;
            left: 50%;
            top: 50%;
            transform: translate(-50%, -50%);
        }

        .pulse .wave {
            position: absolute;
            left: 7%;
            top: 7%;
            width: 86%;
            height: 86%;
            border-radius: 50%;
            opacity: 0;
        }

        .pulse .wave::after {
            content: "";
            position: absolute;
            left: 7%;
            top: 7%;
            width: 86%;
            height: 86%;
            border-radius: 50%;
            opacity: 0;
        }
    }
</style>
