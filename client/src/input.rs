#![allow(unused)]
use crate::{
    context::GameContext,
    state_manager::{State, Transition},
};
use clipboard::{ClipboardContext, ClipboardProvider};
use unicode_names2::name;
use winit::event::VirtualKeyCode;

use winit_input_helper::TextChar;

type InputCallback = fn(&mut GameContext, String) -> anyhow::Result<Transition<GameContext>>;

pub struct Input {
    characters: Vec<char>,
    index: usize,
    callback: Option<InputCallback>,
    interrupt: bool,
    title: Option<String>,
}
impl Input {
    pub fn new() -> Self {
        Self {
            characters: vec![],
            index: 0,
            callback: None,
            interrupt: true,
            title: None,
        }
    }
    pub fn with_default_string(mut self, value: String) -> Self {
        self.characters.extend(value.chars());
        self
    }
    pub fn set_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
    pub fn set_interrupt(mut self, i: bool) -> Self {
        self.interrupt = i;
        self
    }
    pub fn set_callback(mut self, callback: InputCallback) -> Self {
        self.callback = Some(callback);
        self
    }
    pub fn speak(&self, index: usize, ctx: &mut GameContext) -> anyhow::Result<()> {
        let spoken: String;
        if self.characters.is_empty() || !(index < self.characters.len()) {
            spoken = String::from("blank");
        } else {
            spoken = self.get_name(self.characters[index]);
        }
        ctx.speaker.speak(spoken, self.interrupt)?;
        Ok(())
    }
    pub fn next(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.index < self.characters.len() {
            self.index += 1;
            self.speak(self.index, ctx)?;
        } else if self.index == self.characters.len() {
            self.speak(self.index, ctx)?;
        }
        Ok(())
    }
    pub fn previous(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.index > 0 {
            self.index -= 1;
            self.speak(self.index, ctx)?;
        } else if self.index == 0 {
            self.speak(self.index, ctx)?;
        }
        Ok(())
    }
    pub fn speak_string(&self, ctx: &mut GameContext) -> anyhow::Result<()> {
        let text: String = self.characters.iter().collect();
        ctx.speaker.speak(text, self.interrupt)?;
        Ok(())
    }
    pub fn delete(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.characters.is_empty() {
            return Ok(());
        }
        if self.index > 0 {
            let text = self.characters.remove(self.index - 1);
            ctx.speaker
                .speak(format!("{} Deleted", self.get_name(text)), self.interrupt)?;
            self.index -= 1;
        }
        Ok(())
    }
    pub fn on_activate(
        &self,
        ctx: &mut GameContext,
        escape_pressed: bool,
    ) -> anyhow::Result<Transition<GameContext>> {
        if let Some(cb) = &self.callback {
            let mut text = String::from("");
            if !escape_pressed {
                text = self.characters.iter().collect();
            }
            return cb(ctx, text);
        }
        Ok(Transition::None)
    }
    pub fn get_name(&self, text: char) -> String {
        if text == ' ' {
            return "Space".to_string();
        } else if text.is_ascii_alphabetic() {
            if text.is_ascii_uppercase() {
                return format!("Capital {}", text);
            }
            return text.to_string();
        } else {
            if text.is_ascii_punctuation() {
                if let Some(txt) = unicode_names2::name(text) {
                    return txt.to_string();
                }
            }
        }
        text.to_string()
    }
}

impl State<GameContext> for Input {
    fn on_push(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if let Some(title) = &self.title {
            ctx.speaker.speak(title, self.interrupt)?;
        }
        Ok(())
    }
    fn on_update(
        &mut self,
        ctx: &mut GameContext,
        depth: usize,
    ) -> anyhow::Result<Transition<GameContext>> {
        if depth != 0 {
            return Ok(Transition::None);
        }
        if ctx.input.held_control() {
            if ctx.input.key_pressed(VirtualKeyCode::V) {
                let mut context: ClipboardContext = ClipboardProvider::new().unwrap();
                let content = context.get_contents().unwrap();
                ctx.speaker
                    .speak(format!("{} Pasted", content), self.interrupt)?;
                for i in content.chars() {
                    self.characters.insert(self.index, i);
                    self.index += 1;
                }
                return Ok(Transition::None);
            } else if ctx.input.key_pressed(VirtualKeyCode::C) {
                let mut context: ClipboardContext = ClipboardProvider::new().unwrap();
                context
                    .set_contents(self.characters.iter().collect())
                    .unwrap();
                ctx.speaker.speak("Copyed", self.interrupt)?;
                return Ok(Transition::None);
            }
        }
        if ctx.input.key_pressed_os(VirtualKeyCode::Back) {
            self.delete(ctx)?;
        } else if ctx.input.key_pressed_os(VirtualKeyCode::Left) {
            self.previous(ctx)?;
        } else if ctx.input.key_pressed_os(VirtualKeyCode::Right) {
            self.next(ctx)?;
        } else if ctx.input.key_pressed_os(VirtualKeyCode::Up)
            || ctx.input.key_pressed_os(VirtualKeyCode::Down)
        {
            self.speak_string(ctx)?;
        } else if ctx.input.key_pressed(VirtualKeyCode::Return) {
            return self.on_activate(ctx, false);
        } else if ctx.input.key_pressed(VirtualKeyCode::Escape) {
            return self.on_activate(ctx, true);
        } else if ctx.input.key_pressed(VirtualKeyCode::F1) {
            if !ctx.config.data.contains_key("input_speak") {
                ctx.config
                    .data
                    .insert(String::from("input_speak"), String::new());
                ctx.speaker.speak("Typed character speak off", true)?;
            } else {
                ctx.config.data.remove("input_speak");
                ctx.speaker.speak("Typed character speak on", true)?;
            }
        } else {
            for i in ctx.input.text() {
                match i {
                    TextChar::Char(c) => {
                        if c != '\t' {
                            self.characters.insert(self.index, c);
                            self.index += 1;
                            if !ctx.config.data.contains_key("input_speak") {
                                self.speak(self.index - 1, ctx)?;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(Transition::None)
    }
}
