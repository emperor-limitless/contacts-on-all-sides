use reqwest::blocking::Response;
use std::io::{self, Read};

pub struct UrlStream {
    body: Response,
}

impl UrlStream {
    pub fn new(url: &str) -> anyhow::Result<UrlStream> {
        Ok(Self {
            body: reqwest::blocking::get(url)?,
        })
    }
}
impl synthizer::CloseStream for UrlStream {
    fn close(&mut self) -> Result<(), Box<dyn std::fmt::Display>> {
        Ok(())
    }
}

impl Read for UrlStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.body.read(buf)
    }
}
