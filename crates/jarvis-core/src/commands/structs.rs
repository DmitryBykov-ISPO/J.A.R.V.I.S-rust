use std::path::PathBuf;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct JCommandsList {
    #[serde(skip)]
    pub path: PathBuf,

    pub commands: Vec<JCommand>,
}



#[derive(Deserialize, Debug, Clone)]
pub struct JCommand {
    pub id: String,
    pub action: String,
    
    #[serde(default)]
    pub description: String,
    
    #[serde(default)]
    pub exe_path: String,
    
    #[serde(default)]
    pub exe_args: Vec<String>,
    
    #[serde(default)]
    pub cli_cmd: String,
    
    #[serde(default)]
    pub cli_args: Vec<String>,
    
    #[serde(default)]
    pub sounds: Vec<String>,
    
    pub phrases: Vec<String>,
}
