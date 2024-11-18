use crate::{response, EXIT_CODE_0};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config(IO({0:?}))")]
    Io(#[from] std::io::Error),

    #[error("Config(Json({0:?}))")]
    Json(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, ConfigError>;

pub fn read() -> Result<Config> {
    let loc = get_config_location();

    let content = fs::read_to_string(loc)?;

    Ok(serde_json::from_str(&content)?)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub tick_rate: u64,
    pub log: LogConfig,
    pub email: EmailConfig,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogConfig {
    pub level: String,
    pub dir: Option<String>,
    pub stdout: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub recipients: Vec<String>,
    pub from: String,
    pub server: String,
    pub domain: String,
    pub port: u16,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Srfax {
    pub name: String,
    pub access_id: String,
    pub access_pwd: String,
    pub file_dir: String,
    pub download_fmt: response::DownloadFormat,
    pub delete_after: bool,
}

lazy_static! {
    pub static ref CONFIG: Config = unwrap!(read());
}

pub fn check_config_exists() -> Result<()> {
    let loc = get_config_location();
    if loc.exists() {
        // all good
        Ok(())
    } else {
        // write file and exit
        println!("config does not exist, writing and exiting");
        write_default_config(&loc)?;

        check_srfaxes_exists()?;
        std::process::exit(EXIT_CODE_0);
    }
}

pub fn write_default_config(path: &Path) -> Result<()> {
    let config = Config {
        tick_rate: 5, // in seconds
        log: LogConfig {
            level: "info".to_string(),
            dir: None,
            stdout: true,
        },
        email: EmailConfig {
            enabled: false,
            recipients: vec![],
            from: String::new(),
            server: "127.0.0.1".to_string(),
            domain: String::new(),
            port: 25,
        },
    };
    let config_content = serde_json::to_string_pretty(&config)?;

    let mut file = File::create(&path)?;
    file.write_all(config_content.as_bytes())?;
    Ok(())
}

pub fn check_srfaxes_exists() -> Result<()> {
    let loc = get_srfax_location();
    if loc.exists() {
        Ok(())
    } else {
        println!("srfaxes does not exist, writing and exiting");

        write_default_srfaxes(&loc)?;

        std::process::exit(EXIT_CODE_0);
    }
}

pub fn write_default_srfaxes(path: &Path) -> Result<()> {
    let srfaxes = vec![Srfax {
        name: "Example 1".to_string(),
        access_id: String::new(),
        access_pwd: String::new(),
        file_dir: "srfax1".to_string(),
        download_fmt: response::DownloadFormat::PDF,
        delete_after: false,
    }];

    let content = serde_json::to_string_pretty(&srfaxes)?;

    fs::write(path, content)?;

    Ok(())
}

pub fn get_config_location() -> PathBuf {
    Path::new("config.json").to_path_buf()
}

pub fn get_srfax_location() -> PathBuf {
    Path::new("srfaxes.json").to_path_buf()
}

pub fn get_srfaxes() -> Result<Vec<Srfax>> {
    let path = get_srfax_location();

    let content = fs::read_to_string(path)?;

    let srfaxes: Vec<Srfax> = serde_json::from_str(&content)?;

    Ok(srfaxes)
}
