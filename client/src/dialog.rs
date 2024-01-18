use crate::{
    context::GameContext,
    state_manager::{State, Transition},
};
use winit::event::VirtualKeyCode;

pub struct Dialog {
    text: String,
    index: usize,
}
impl Dialog {
    pub fn new(text: String) -> Self {
        Self { text, index: 0 }
    }
    pub fn speak(&self, ctx: &mut GameContext) -> anyhow::Result<()> {
        let lines = self.text.lines().collect::<Vec<&str>>();
        ctx.sound.play("sounds/notifications/dialog.mp3", false);
        ctx.speaker.speak(lines[0], true)?;
        Ok(())
    }
}
impl State<GameContext> for Dialog {
    fn on_push(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        self.speak(ctx)?;
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
        if ctx.input.key_pressed_os(VirtualKeyCode::Left)
            || ctx.input.key_pressed_os(VirtualKeyCode::Right)
            || ctx.input.key_pressed_os(VirtualKeyCode::Up)
            || ctx.input.key_pressed_os(VirtualKeyCode::Down)
        {
            let lines = self.text.lines().collect::<Vec<&str>>();
            ctx.speaker.speak(lines[self.index], true)?;
        }
        if ctx.input.key_pressed(VirtualKeyCode::Return) {
            let lines = self.text.lines().collect::<Vec<&str>>();
            if self.index == lines.len() - 1 {
                return Ok(Transition::Pop(1));
            }
            self.index += 1;
            self.speak(ctx)?;
        }
        Ok(Transition::None)
    }
}
