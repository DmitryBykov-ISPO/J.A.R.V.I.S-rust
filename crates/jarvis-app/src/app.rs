use std::time::SystemTime;

use jarvis_core::{audio, audio_processing, commands, config, listener, recorder, stt, COMMANDS_LIST, intent};
use rand::prelude::*;

pub fn start() -> Result<(), ()> {
    // start the loop
    main_loop()
}

fn main_loop() -> Result<(), ()> {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let mut start: SystemTime;
    let sounds_directory = audio::get_sound_directory().unwrap();
    let frame_length: usize = 512; // default for every wake-word engine
    let mut frame_buffer: Vec<i16> = vec![0; frame_length];
    let mut silence_frames: u32 = 0;

    // play some run phrase
    // @TODO. Different sounds? Or better make it via commands or upcoming events system.
    audio::play_sound(&sounds_directory.join("run.wav"));

    // start recording
    match recorder::start_recording() {
        Ok(_) => info!("Recording started."),
        Err(_) => {
            error!("Cannot start recording.");
            return Err(()); // quit
        }
    }

    // the loop
    'wake_word: loop {
        // read from microphone
        recorder::read_microphone(&mut frame_buffer);

        // process audio (gain -> noise suppression -> VAD)
        let processed = audio_processing::process(&frame_buffer);

        // skip if no voice detected (vad)
        if !processed.is_voice {
            continue 'wake_word;
        }

        // recognize wake-word
        match listener::data_callback(&frame_buffer) {
            Some(_keyword_index) => {
                // reset some things
                stt::reset_wake_recognizer();
                stt::reset_speech_recognizer();
                audio_processing::reset();

                // wake-word activated, process further commands
                // capture current time
                start = SystemTime::now();
                silence_frames = 0;

                // play some greet phrase
                // @TODO. Make it via commands or upcoming events system.
                audio::play_sound(&sounds_directory.join(format!(
                        "{}.wav",
                        config::ASSISTANT_GREET_PHRASES
                            .choose(&mut rand::thread_rng())
                            .unwrap()
                    )));

                // wait for voice commands
                'voice_recognition: loop {
                    // read from microphone
                    recorder::read_microphone(&mut frame_buffer);

                    // process first
                    let processed = audio_processing::process(&frame_buffer);

                    // detect silence, return to wake-word if silence
                    if processed.is_voice {
                        silence_frames = 0;
                    } else {
                        silence_frames += 1;
                        if silence_frames > config::VAD_SILENCE_FRAMES * 2 {
                            info!("Long silence detected, returning to wake word mode.");
                            break 'voice_recognition;
                        }
                    }

                    // stt part (without partials)
                    if let Some(mut recognized_voice) = stt::recognize(&frame_buffer, false) {
                        // something was recognized
                        info!("Recognized voice: {}", recognized_voice);

                        // filter recognized voice
                        // @TODO. Better recognized voice filtration.
                        recognized_voice = recognized_voice.to_lowercase();

                        // answer again if it's activation phrase repeated
                        if recognized_voice.contains(config::VOSK_FETCH_PHRASE) {
                            info!("Wake word detected during chaining, reactivating...");
                            
                            // play greet sound
                            audio::play_sound(&sounds_directory.join(format!(
                                "{}.wav",
                                config::ASSISTANT_GREET_PHRASES
                                    .choose(&mut rand::thread_rng())
                                    .unwrap()
                            )));
                            
                            // reset timer and continue listening
                            start = SystemTime::now();
                            silence_frames = 0;
                            stt::reset_speech_recognizer();
                            continue 'voice_recognition;
                        }

                        // filter out activation phrase from command
                        for tbr in config::ASSISTANT_PHRASES_TBR {
                            recognized_voice = recognized_voice.replace(tbr, "");
                        }
                        recognized_voice = recognized_voice.trim().into();

                        // skip if nothing left after filtering (*evil laugh*)
                        if recognized_voice.is_empty() {
                            continue 'voice_recognition;
                        }

                        // infer command (try intent recognition first, fallback to levenshtein)
                        let cmd_result = if let Some((intent_id, confidence)) = 
                            rt.block_on(intent::classify(&recognized_voice)) 
                        {
                            info!("Intent recognized: {} (confidence: {:.2})", intent_id, confidence);
                            intent::get_command_by_intent(COMMANDS_LIST.get().unwrap(), &intent_id)
                        } else {
                            info!("Intent not recognized, trying levenshtein fallback ...");
                            commands::fetch_command(&recognized_voice, COMMANDS_LIST.get().unwrap())
                        };

                        if let Some((cmd_path, cmd_config)) = cmd_result {
                            info!("Command found: {:?}", cmd_path);
                            info!("Executing!");

                            // execute the command
                            match commands::execute_command(&cmd_path, &cmd_config) {
                                Ok(chain) => {
                                    // success
                                    info!("Command executed successfully.");

                                    if chain {
                                        // chain commands
                                        start = SystemTime::now();
                                    } else {
                                        // skip, if chaining is not required
                                        start = start
                                            .checked_sub(core::time::Duration::from_secs(1000))
                                            .unwrap();
                                    }

                                    continue 'voice_recognition; // continue voice recognition
                                }
                                Err(msg) => {
                                    // fail
                                    error!("Error executing command: {}", msg);
                                }
                            }
                        }

                        // return to wake-word listening after command execution (no matter successful or not)
                        break 'voice_recognition;
                    }

                    // only recognize voice for a certain period of time
                    match start.elapsed() {
                        Ok(elapsed) if elapsed > config::CMS_WAIT_DELAY => {
                            // return to wake-word listening after N seconds
                            break 'voice_recognition;
                        }
                        _ => (),
                    }

                    // reset things
                    stt::reset_wake_recognizer();
                    audio_processing::reset();
                }
            }
            None => (),
        }
    }

    Ok(())
}

fn keyword_callback(keyword_index: i32) {}

pub fn close(code: i32) {
    info!("Closing application.");
    std::process::exit(code);
}
