#![allow(unused)]
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;
pub struct Dm {
    fin: String,
    f: fernet::Fernet,
    pub m: HashMap<String, String>,
}
impl Dm {
    pub fn new(fin: String, key: String) -> Self {
        Self {
            fin,
            f: fernet::Fernet::new(&key).unwrap(),
            m: HashMap::new(),
        }
    }
    pub fn encrypt(&mut self, text: &str) -> String {
        let entx = self.f.encrypt(text.as_bytes());
        entx
    }
    pub fn encrypto(&mut self, text: &[u8]) -> String {
        let entx = self.f.encrypt(text);
        entx
    }

    pub fn decrypt(&mut self, text: &str) -> Vec<u8> {
        let detx = self.f.decrypt(&text).unwrap();
        detx
    }
    pub fn decrypto(&mut self, text: &[u8]) -> anyhow::Result<Vec<u8>> {
        let detx = self.f.decrypt(&String::from_utf8(text.to_vec())?)?;
        Ok(detx)
    }
    pub fn exists(&self, item: String) -> bool {
        self.m.contains_key(&item)
    }
    pub fn get(&mut self, value: &str) -> String {
        if !self.m.contains_key(value) {
            return "".to_string();
        }
        self.m.get(value).unwrap().to_string()
    }
    pub fn add(&mut self, item: String, value: String) {
        self.m.insert(item, value);
    }
    pub fn save(&mut self) -> anyhow::Result<()> {
        if self.m.len() == 0 {
            return Ok(());
        }
        let pc = Path::new(&self.fin);
        if !pc.exists() {
            File::create(&self.fin)?;
        }

        let hm = self
            .f
            .encrypt(serde_json::to_string(&self.m).unwrap().as_bytes());
        fs::write(&self.fin.to_string(), hm)?;
        Ok(())
    }
    pub fn load(&mut self) {
        let pc = Path::new(&self.fin);
        if !pc.exists() {
            return;
        }

        self.m = serde_json::from_str(
            std::str::from_utf8(
                &self
                    .f
                    .decrypt(std::str::from_utf8(&fs::read(&self.fin).unwrap()).unwrap())
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
