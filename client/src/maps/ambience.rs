use crate::{audio::sound::Sound, context::GameContext};
pub struct Ambience {
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
    sound: Sound,
    is_idle: bool,
}
impl Ambience {
    pub fn new(
        min_x: isize,
        max_x: isize,
        min_y: isize,
        max_y: isize,
        sound_path: String,
        sound_volume: f64,
        ctx: &mut GameContext,
    ) -> anyhow::Result<Self> {
        let s = Self {
            min_x,
            max_x,
            min_y,
            max_y,
            sound: ctx.sound.play(&sound_path, true),
            is_idle: true,
        };
        let mut volume = sound_volume;
        if volume < 0.0 {
            volume = 0.0;
        }
        if volume > 1.0 {
            volume = 1.0;
        }
        s.sound.src.gain().set(volume)?;
        s.sound.src.pause()?;
        Ok(s)
    }
    pub fn in_range(&self, x: isize, y: isize) -> bool {
        if x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y {
            return true;
        }
        false
    }
    pub fn sound_loop(&mut self, x: isize, y: isize, _ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.in_range(x, y) {
            self.sound.src.play()?;
            self.is_idle = false;
        } else {
            self.sound.src.pause()?;
            self.is_idle = true;
        }
        Ok(())
    }
}
