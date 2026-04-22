use std::io::{self, Write};

use jarvis_core::llm::{ChatMessage, LlmClient};
use jarvis_core::{COMMANDS_LIST, DB, JCommandsList, commands, config, db, intent};

const ASK_MAX_TOKENS: u32 = 512;

fn print_help() {
    println!("
--## Jarvis CLI - Testing Tool ##--

Commands:
  classify <text>    - Test intent classification
  execute <text>     - Simulate voice input and execute command
  ask <prompt>       - Send prompt to Groq LLM and print reply
  list               - List all loaded commands
  phrases            - List all training phrases
  hash               - Show commands hash
  settings           - Dump all settings
  help               - Show this help
  exit               - Exit the CLI
");
}

fn list_commands(commands: &[JCommandsList]) {
    println!("\n[ Loaded Commands ]");
    for cmd_list in commands {
        println!("  📁 {}", cmd_list.path.display());
        for cmd in &cmd_list.commands {
            println!("     ├─ id: {}", cmd.id);
            println!("     ├─ type: {}", cmd.cmd_type);
            println!("     └─ phrases: {} languages", cmd.phrases.len());
        }
    }
    println!();
}

fn list_phrases(commands: &[JCommandsList]) {
    println!("\n[ Training Phrases ]");
    for cmd_list in commands {
        for cmd in &cmd_list.commands {
            println!("  [{}]", cmd.id);
            for (lang, phrases) in &cmd.phrases {
                println!("    lang: {}", lang);
                for phrase in phrases {
                    println!("      - {}", phrase);
                }
            }
        }
    }
    println!();
}

async fn classify_text(text: &str) {
    match intent::classify(text).await {
        Some((intent_id, confidence)) => {
            println!("  ✓ Intent: {} (confidence: {:.2}%)", intent_id, confidence * 100.0);
        }
        None => {
            println!("  ✗ No intent matched (below threshold)");
        }
    }
}

async fn execute_text(commands: &[JCommandsList], text: &str) {
    // try intent classification first
    if let Some((intent_id, confidence)) = intent::classify(text).await {
        println!("  Intent: {} (confidence: {:.2}%)", intent_id, confidence * 100.0);
        
        if let Some((cmd_path, cmd)) = intent::get_command_by_intent(commands, &intent_id) {
            println!("  Command: {:?}", cmd_path);
            println!("  Type: {}", cmd.cmd_type);
            println!("  Executing...");
            
            match commands::execute_command(cmd_path, cmd, Some(text), None) {
                Ok(chain) => println!("  ✓ Success (chain: {})", chain),
                Err(e) => println!("  ✗ Error: {}", e),
            }
            return;
        }
    }
    
    // fallback to levenshtein
    println!("  Intent not matched, trying levenshtein fallback...");
    if let Some((cmd_path, cmd)) = commands::fetch_command(text, commands) {
        println!("  Command: {:?}", cmd_path);
        println!("  Type: {}", cmd.cmd_type);
        println!("  Executing...");
        
        match commands::execute_command(cmd_path, cmd, Some(text), None) {
            Ok(chain) => println!("  ✓ Success (chain: {})", chain),
            Err(e) => println!("  ✗ Error: {}", e),
        }
    } else {
        println!("  ✗ No command matched");
    }
}

fn ask_llm(prompt: &str) -> i32 {
    let client = match LlmClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("LLM not configured: {}", e);
            eprintln!("Set GROQ_TOKEN (and optionally GROQ_BASE_URL, GROQ_MODEL).");
            return 2;
        }
    };
    let messages = [ChatMessage::user(prompt)];
    match client.complete(&messages, ASK_MAX_TOKENS) {
        Ok(reply) => {
            println!("{}", reply);
            0
        }
        Err(e) => {
            eprintln!("LLM request failed: {}", e);
            1
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() >= 2 && argv[1] == "ask" {
        if argv.len() < 3 {
            eprintln!("Usage: jarvis-cli ask <prompt>");
            std::process::exit(2);
        }
        let prompt = argv[2..].join(" ");
        std::process::exit(ask_llm(&prompt));
    }

    // init logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    println!("Jarvis CLI v{}", config::APP_VERSION.unwrap_or("unknown"));

    // init dirs
    config::init_dirs()?;
    
    // init settings
    let settings = db::init();
    DB.set(settings.arc().clone())
        .expect("DB already initialized");

    // parse commands
    println!("\n[*] Loading commands...");
    let cmds = match commands::parse_commands() {
        Ok(c) => {
            println!("    Loaded {} command groups", c.len());
            c
        }
        Err(e) => {
            println!("    Warning: {}", e);
            Vec::new()
        }
    };
    COMMANDS_LIST.set(cmds).expect("Failed to set commands list");
    
    // init intent classifier
    println!("[*] Initializing intent classifier...");
    match intent::init(COMMANDS_LIST.get().unwrap()).await {
        Ok(_) => println!("    Intent classifier ready"),
        Err(e) => println!("    Warning: {}", e),
    }
    
    // init sound
    println!("[*] Initializing audio...");
    if let Err(e) = jarvis_core::audio::init() {
        println!("    Warning: Audio init failed: {:?}", e);
    }

    print_help();

    // REPL loop
    let mut input = String::new();
    loop {
        print!("jarvis> ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");
        
        match cmd {
            "exit" | "quit" | "q" => {
                println!("Bye!");
                break;
            }
            "help" | "h" | "?" => print_help(),
            "list" | "ls" => list_commands(COMMANDS_LIST.get().unwrap()),
            "phrases" => list_phrases(COMMANDS_LIST.get().unwrap()),
            "hash" => {
                let hash = commands::commands_hash(COMMANDS_LIST.get().unwrap());
                println!("  Commands hash: {}", hash);
            }
            "settings" => {
                println!("\n[ Current Settings ]");
                for (key, val) in settings.dump() {
                    println!("  {} = {}", key, val);
                }
                println!();
            }
            "classify" | "c" => {
                if arg.is_empty() {
                    println!("  Usage: classify <text>");
                } else {
                    classify_text(arg).await;
                }
            }
            "execute" | "exec" | "e" => {
                if arg.is_empty() {
                    println!("  Usage: execute <text>");
                } else {
                    execute_text(COMMANDS_LIST.get().unwrap(), arg).await;
                }
            }
            "ask" | "a" => {
                if arg.is_empty() {
                    println!("  Usage: ask <prompt>");
                } else {
                    ask_llm(arg);
                }
            }
            "reload" => {
                println!("  Note: Reload requires app restart (statics can't be reset)");
            }
            _ => {
                // treat unknown commands as text to classify
                classify_text(input).await;
            }
        }
    }
    
    Ok(())
}
