#![allow(unused)]
use super::sound::{Sound, SoundManager};
use crate::context::GameContext;
use winit::event::VirtualKeyCode;

use anyhow::format_err;

pub struct Music {
    sound: Option<Sound>,
}

impl Music {
    pub fn new() -> Self {
        Self { sound: None }
    }
    pub fn play(&mut self, filename: &str, sound_manager: &mut SoundManager) {
        self.sound = Some(sound_manager.play(filename, true));
    }
    pub fn pause(&self) -> anyhow::Result<()> {
        if let Some(snd) = &self.sound {
            snd.src.pause()?;
        }
        Ok(())
    }
    pub fn resume(&self) -> anyhow::Result<()> {
        if let Some(snd) = &self.sound {
            snd.src.play()?;
        }
        Ok(())
    }
    pub fn get_volume(&self) -> anyhow::Result<f64> {
        if let Some(snd) = &self.sound {
            let val = snd.src.gain().get()?;
            return Ok(val);
        }
        Err(format_err!("Music.get_volume: Instance uninitialized!"))
    }
    pub fn set_volume(&mut self, volume: f64) -> anyhow::Result<()> {
        if let Some(snd) = &mut self.sound {
            snd.src.gain().set(volume)?;
            return Ok(());
        }
        Err(format_err!("Music.set_volume: Instance uninitialized!"))
    }
    fn volume_loop(&mut self, ctx: &mut GameContext) {
        if self.sound.is_none() {
            return;
        }
        let volume = self.get_volume().unwrap();
        if ctx.input.key_pressed_os(VirtualKeyCode::PageDown) && volume > 0.1 {
            self.set_volume(volume - 0.1).unwrap();
        } else if ctx.input.key_pressed_os(VirtualKeyCode::PageUp) && volume < 1.0 {
            self.set_volume(volume + 0.1).unwrap();
        }
    }
}
