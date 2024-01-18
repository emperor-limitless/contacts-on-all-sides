use std::string::ToString;
use winit::event::VirtualKeyCode;

use crate::{
    audio::sound::Sound,
    context::GameContext,
    state_manager::{State, Transition},
};

pub type MenuItemCallback<MCtx> =
    fn(&mut MenuItem<MCtx>, &mut GameContext, &mut MCtx) -> anyhow::Result<Transition<GameContext>>;

pub struct MenuItem<MCtx> {
    pub text: String,
    activate_callback: Option<MenuItemCallback<MCtx>>,
}

impl<MCtx> MenuItem<MCtx> {
    pub fn new<S: ToString>(text: S) -> Self {
        Self {
            text: text.to_string(),
            activate_callback: None,
        }
    }

    pub fn on_activate(mut self, activate_callback: MenuItemCallback<MCtx>) -> Self {
        self.activate_callback = Some(activate_callback);
        self
    }

    pub fn speak(
        &self,
        ctx: &mut GameContext,
        pos: usize,
        len: usize,
    ) -> Result<Option<tts::UtteranceId>, tts::Error> {
        ctx.speaker
            .speak(format!("{}. {} of {}", self.text, pos, len), true)
    }
}

pub struct MenuBuilder<MCtx> {
    title: String,
    items: Vec<MenuItem<MCtx>>,
    menu_context: MCtx,
    music: Option<String>,
}

impl<MCtx> MenuBuilder<MCtx> {
    pub fn new<S: ToString>(title: S, menu_context: MCtx) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
            menu_context,
            music: None,
        }
    }

    pub fn build(self) -> Menu<MCtx> {
        Menu {
            title: self.title,
            items: self.items,
            menu_context: self.menu_context,
            selected: None,
            music: self.music,
            music_handle: None,
        }
    }
    pub fn with_music(mut self, filename: &str) -> Self {
        self.music = Some(filename.into());
        self
    }
    pub fn item(mut self, item: MenuItem<MCtx>) -> Self {
        self.items.push(item);
        self
    }
}

pub struct Menu<MCtx> {
    title: String,
    items: Vec<MenuItem<MCtx>>,
    menu_context: MCtx,
    selected: Option<usize>,
    music: Option<String>,
    music_handle: Option<Sound>,
}

impl<MCtx> Menu<MCtx> {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    fn set_volume(&mut self, volume: f64, ctx: &mut GameContext) -> anyhow::Result<()> {
        if let Some(snd) = &self.music_handle {
            snd.src.gain().set(volume)?;
            ctx.config
                .data
                .insert("menu_music_volume".to_string(), volume.to_string());
        }
        Ok(())
    }
    fn get_volume(&mut self) -> anyhow::Result<f64> {
        if let Some(snd) = &self.music_handle {
            let vol = snd.src.gain().get()?;
            return Ok(vol);
        }
        Ok(0.0)
    }
    fn music_loop(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.music_handle.is_none() {
            return Ok(());
        }
        let volume = self.get_volume().unwrap();
        if ctx.input.key_pressed_os(VirtualKeyCode::PageDown) && volume > 0.1 {
            self.set_volume(volume - 0.1, ctx).unwrap();
        } else if ctx.input.key_pressed_os(VirtualKeyCode::PageUp) && volume < 1.0 {
            self.set_volume(volume + 0.1, ctx).unwrap();
        }
        Ok(())
    }
    fn next_item(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        let selected = if let Some(idx) = self.selected {
            (idx + 1) % self.items.len()
        } else {
            0
        };
        self.items[selected].speak(ctx, selected + 1, self.items.len())?;
        ctx.sound.play("sounds/menus/main/move.mp3", false);
        self.selected = Some(selected);
        Ok(())
    }

    fn previous_item(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        let selected = if let Some(idx) = self.selected {
            (idx + self.items.len() - 1) % self.items.len()
        } else {
            self.items.len() - 1
        };
        self.items[selected].speak(ctx, selected + 1, self.items.len())?;
        ctx.sound.play("sounds/menus/main/move.mp3", false);
        self.selected = Some(selected);
        Ok(())
    }

    fn activate_item(&mut self, ctx: &mut GameContext) -> anyhow::Result<Transition<GameContext>> {
        let Some(selected) = self.selected else { return Ok(Transition::None); };
        let Some(cb) = self.items[selected].activate_callback else { return Ok(Transition::None); };
        ctx.sound.play("sounds/menus/main/enter.mp3", false);
        cb(&mut self.items[selected], ctx, &mut self.menu_context)
    }
}

impl<MCtx> State<GameContext> for Menu<MCtx> {
    fn on_push(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        ctx.speaker.speak(
            format!(
                "{} Menu. {} item{}",
                self.title,
                self.items.len(),
                if self.items.len() > 1 { 's' } else { ' ' }
            ),
            true,
        )?;
        if let Some(mus) = &self.music {
            let music_handle = ctx.sound.play(mus, true);
            self.music_handle = Some(music_handle);
            if let Some(vol) = ctx.config.data.get("menu_music_volume") {
                self.set_volume(vol.parse::<f64>().unwrap(), ctx)?;
            }
        }
        Ok(())
    }
    fn on_focus(&mut self, _ctx: &mut GameContext) -> anyhow::Result<()> {
        if let Some(snd) = &self.music_handle {
            snd.src.play()?;
        }
        Ok(())
    }
    fn on_unfocus(&mut self, _ctx: &mut GameContext) -> anyhow::Result<()> {
        if let Some(snd) = &self.music_handle {
            snd.src.pause()?;
            snd.gen.playback_position().set(0.0)?;
        }
        Ok(())
    }

    fn on_update(
        &mut self,
        ctx: &mut GameContext,
        depth: usize,
    ) -> anyhow::Result<Transition<GameContext>> {
        // If we're not the active state, return immediately
        if depth != 0 {
            return Ok(Transition::None);
        }
        if ctx.input.key_pressed(VirtualKeyCode::LControl) || ctx.input.key_pressed(VirtualKeyCode::RControl) {
            ctx.speaker.stop()?;
        }

        if ctx.input.close_requested() || ctx.input.key_pressed(VirtualKeyCode::Escape) {
            return Ok(Transition::Pop(1));
        }

        // Input
        if ctx.input.key_pressed_os(VirtualKeyCode::Down) {
            self.next_item(ctx)?;
        }
        if ctx.input.key_pressed_os(VirtualKeyCode::Up) {
            self.previous_item(ctx)?;
        }
        if ctx.input.key_pressed(VirtualKeyCode::Return) {
            return self.activate_item(ctx);
        }
        self.music_loop(ctx)?;
        Ok(Transition::None)
    }
}
