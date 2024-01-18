use directories::ProjectDirs;
use fernet::Fernet;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

pub struct Config {
    pub data: HashMap<String, String>,
    pub encryptor: Fernet,
}
impl Config {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            encryptor: Fernet::new("Em8ynZO658NhekC48wSOJBLImFb6H7-k3a4s3d7Hepk=").unwrap(),
        }
    }
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(dir) = ProjectDirs::from("org", "arelius", env!("CARGO_PKG_NAME")) {
            let cfg = dir.config_dir();
            if !cfg.exists() {
                fs::create_dir_all(cfg)?;
            }
            let path = format!("{}\\settings.dat", dir.config_dir().to_str().unwrap());
            let cfg2 = Path::new(&path);
            if !cfg2.exists() {
                fs::File::create(cfg2)?;
            }
            let text = serde_json::to_string_pretty(&self.data)?;
            let text = self.encryptor.encrypt(text.as_bytes());
            fs::write(cfg2, text)?;
        }
        Ok(())
    }
    pub fn load(&mut self) -> anyhow::Result<()> {
        if let Some(dir) = ProjectDirs::from("org", "arelius", env!("CARGO_PKG_NAME")) {
            let path = format!("{}\\settings.dat", dir.config_dir().to_str().unwrap());
            let cfg = Path::new(&path);
            if !cfg.exists() {
                return Ok(());
            }
            let mut file = fs::File::open(cfg)?;
            let mut text = String::new();
            file.read_to_string(&mut text)?;
            text = String::from_utf8(self.encryptor.decrypt(&text)?)?;
            let deserialized: HashMap<String, String> = serde_json::from_str(&text)?;
            self.data = deserialized;
        }
        Ok(())
    }
}
