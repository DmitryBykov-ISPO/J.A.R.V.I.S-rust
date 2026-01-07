use crate::APP_DIR;
use rand::prelude::*;
use seqdiff::ratio;
// use serde_yaml;
use std::path::Path;
use std::{fs, fs::File};

use core::time::Duration;
use std::path::PathBuf;
use std::process::{Child, Command};
// use tauri::Manager;

mod structs;
pub use structs::*;

use std::collections::HashMap;

use crate::{config};

// @TODO. Allow commands both in yaml and json format.
pub fn parse_commands() -> Result<Vec<JCommandsList>, String> {
    // collect commands
    let mut commands: Vec<JCommandsList> = Vec::new();

    let commands_path = APP_DIR.join(config::COMMANDS_PATH);
    let cmd_dirs = fs::read_dir(&commands_path)
        .map_err(|e| format!("Error reading commands directory {:?}: {}", commands_path, e))?;

    for entry in cmd_dirs {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read command directory entry: {}", e);
                continue;
            }
        };
        
        let cmd_path = entry.path();
        let toml_file = cmd_path.join("command.toml");
        
        if !toml_file.exists() {
            continue;
        }
        
        // read and parse TOML
        let content = match fs::read_to_string(&toml_file) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read {}: {}", toml_file.display(), e);
                continue;
            }
        };

        let file: JCommandsList = match toml::from_str(&content) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to parse {}: {}", toml_file.display(), e);
                continue;
            }
        };

        commands.push(JCommandsList {
            path: cmd_path,
            commands: file.commands,
        });
    }

    if commands.is_empty() {
        Err("No commands found".into())
    } else {
        info!("Loaded {} commands", commands.len());
        Ok(commands)
    }
}


// Commands hash generation for cache invalidation (deterministi c)
pub fn commands_hash(commands: &Vec<JCommandsList>) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    
    // collect all command ids and phrases, sorted
    let mut all_ids: Vec<_> = commands.iter()
        .flat_map(|ac| ac.commands.iter().map(|c| (&c.id, &c.phrases)))
        .collect();
    all_ids.sort_by_key(|(id, _)| *id);
    
    for (id, phrases) in all_ids {
        hasher.update(id.as_bytes());
        for phrase in phrases {
            hasher.update(phrase.as_bytes());
        }
    }
    
    format!("{:x}", hasher.finalize())
}


pub fn fetch_command<'a>(
    phrase: &str,
    commands: &'a Vec<JCommandsList>,
) -> Option<(&'a PathBuf, &'a JCommand)> {
    let mut result: Option<(&PathBuf, &JCommand)> = None;
    let mut best_score = config::CMD_RATIO_THRESHOLD;

    // normalize input
    let phrase = phrase.trim().to_lowercase();
    if phrase.is_empty() {
        return None;
    }

    let phrase_chars: Vec<char> = phrase.chars().collect();
    let phrase_words: Vec<&str> = phrase.split_whitespace().collect();

    for cmd_list in commands {
        for cmd in &cmd_list.commands {
            for cmd_phrase in &cmd.phrases {
                let cmd_phrase = cmd_phrase.trim().to_lowercase();
                let cmd_phrase_chars: Vec<char> = cmd_phrase.chars().collect();
                
                // character-level similarity
                let char_ratio = ratio(&phrase_chars, &cmd_phrase_chars);
                
                // word-level similarity (handles word order)
                let cmd_words: Vec<&str> = cmd_phrase.split_whitespace().collect();
                let word_score = word_overlap_score(&phrase_words, &cmd_words);
                
                // combined score (weighted average)
                let score = (char_ratio * 0.6) + (word_score * 0.4);
                
                // early exit on perfect match
                if score >= 99.0 {
                    debug!("Perfect match: '{}' -> '{}'", phrase, cmd_phrase);
                    return Some((&cmd_list.path, cmd));
                }
                
                if score > best_score {
                    best_score = score;
                    result = Some((&cmd_list.path, cmd));
                }
            }
        }
    }

    if let Some((cmd_path, cmd)) = result {
        info!(
            "Fuzzy match: '{}' -> cmd '{}' (score: {:.1}%)",
            phrase, cmd.id, best_score
        );
        Some((cmd_path, cmd))
    } else {
        debug!("No match for '{}' (best: {:.1}%)", phrase, best_score);
        None
    }
}


fn word_overlap_score(input_words: &[&str], cmd_words: &[&str]) -> f64 {
    if input_words.is_empty() || cmd_words.is_empty() {
        return 0.0;
    }

    let mut matched = 0.0;
    
    for input_word in input_words {
        // find best matching word in command
        let best_word_match = cmd_words
            .iter()
            .map(|cmd_word| {
                let iw: Vec<char> = input_word.chars().collect();
                let cw: Vec<char> = cmd_word.chars().collect();
                ratio(&iw, &cw)
            })
            .fold(0.0_f64, |a, b| a.max(b));
        
        // count as match if word similarity > 70%
        if best_word_match > 70.0 {
            matched += best_word_match / 100.0;
        }
    }

    // normalize by max word count
    let max_words = input_words.len().max(cmd_words.len()) as f64;
    (matched / max_words) * 100.0
}


// @TODO. Rewrite executors by executor type struct. (with match arms)
pub fn execute_exe(exe: &str, args: &Vec<String>) -> std::io::Result<Child> {
    Command::new(exe).args(args).spawn()
}

pub fn execute_cli(cmd: &str, args: &Vec<String>) -> std::io::Result<Child> {
    debug!("Spawning cmd as: cmd /C {} {:?}", cmd, args);

    if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(cmd).args(args).spawn()
    } else {
        Command::new("sh").arg("-c").arg(cmd).args(args).spawn()
    }
}

pub fn execute_command(
    cmd_path: &PathBuf,
    cmd_config: &JCommand,
    // app_handle: &tauri::AppHandle,
) -> Result<bool, String> {
    // let sounds_directory = audio::get_sound_directory().unwrap();

    match cmd_config.action.as_str() {
        "voice" => {
            // VOICE command type
            let random_cmd_sound = format!(
                "{}.wav",
                cmd_config
                    .sounds
                    .choose(&mut rand::thread_rng())
                    .unwrap()
            );
            // events::play(random_cmd_sound, app_handle);
            // audio::play_sound(&sounds_directory.join(random_cmd_sound));

            Ok(true)
        }
        "ahk" => {
            // AutoHotkey command type
            let exe_path_absolute = Path::new(&cmd_config.exe_path);
            let exe_path_local = Path::new(&cmd_path).join(&cmd_config.exe_path);

            if let Ok(_) = execute_exe(
                if exe_path_absolute.exists() {
                    exe_path_absolute.to_str().unwrap()
                } else {
                    exe_path_local.to_str().unwrap()
                },
                &cmd_config.exe_args,
            ) {
                let random_cmd_sound = format!(
                    "{}.wav",
                    cmd_config
                        .sounds
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                );
                // events::play(random_cmd_sound, app_handle);
                // audio::play_sound(&sounds_directory.join(random_cmd_sound));

                Ok(true)
            } else {
                error!("AHK process spawn error (does exe path is valid?)");
                Err("AHK process spawn error (does exe path is valid?)".into())
            }
        }
        "cli" => {
            // CLI command type
            let cli_cmd = &cmd_config.cli_cmd;

            match execute_cli(cli_cmd, &cmd_config.cli_args) {
                Ok(_) => {
                    let random_cmd_sound = format!(
                        "{}.wav",
                        cmd_config
                            .sounds
                            .choose(&mut rand::thread_rng())
                            .unwrap()
                    );
                    // events::play(random_cmd_sound, app_handle);
                    // audio::play_sound(&sounds_directory.join(random_cmd_sound));

                    Ok(true)
                }
                Err(msg) => {
                    error!("CLI command error ({})", msg);
                    Err(format!("Shell command error ({})", msg).into())
                }
            }
        }
        "terminate" => {
            // TERMINATE command type
            let random_cmd_sound = format!(
                "{}.wav",
                cmd_config
                    .sounds
                    .choose(&mut rand::thread_rng())
                    .unwrap()
            );
            // events::play(random_cmd_sound, app_handle);
            // audio::play_sound(&sounds_directory.join(random_cmd_sound));

            std::thread::sleep(Duration::from_secs(2));
            std::process::exit(0);
        }
        "stop_chaining" => {
            // STOP_CHAINING command type
            let random_cmd_sound = format!(
                "{}.wav",
                cmd_config
                    .sounds
                    .choose(&mut rand::thread_rng())
                    .unwrap()
            );
            // events::play(random_cmd_sound, app_handle);
            // audio::play_sound(&sounds_directory.join(random_cmd_sound));

            Ok(false)
        }
        _ => {
            error!("Command type unknown");
            Err("Command type unknown".into())
        }
    }
}

pub fn list(from: &Vec<JCommandsList>) -> Vec<String> {
    let mut out: Vec<String> = vec![];

    for x in from.iter() {
        out.push(String::from(x.path.to_str().unwrap()));
        // out.append()
    }

    out
}
