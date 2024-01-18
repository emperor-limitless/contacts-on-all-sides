#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio;
mod buffer;
mod client;
mod config;
mod context;
mod dialog;
mod dm;
mod game;
mod input;
mod maps;
mod menu;
mod misc;
mod network_state;
mod panic_hook;
mod player;
mod state_manager;
mod stream;
mod timer;
mod url_stream;

use log::error;
use winit::event_loop::EventLoop;

use crate::{
    context::GameContext,
    game::{Game, GameState},
    menu::{MenuBuilder, MenuItem},
    state_manager::{StateManager, Transition},
};
use std::{
    thread,
    time::{Duration, Instant},
};

const FPS: u64 = 60;
const FRAME_TIME: Duration = Duration::from_secs(1 / FPS);
fn run() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(panic_hook::panic_handler));
    let event_loop = EventLoop::<()>::new();
    let mut ctx = GameContext::new(&event_loop)?;
    ctx.config.load()?;
    let mut states = StateManager::<GameContext>::new(4);
    let main_menu = MenuBuilder::new("Main", ())
        .item(MenuItem::new("Login").on_activate(|_, _, _| {
            let mut game = Game::new()?;
            game.state = GameState::LoggingIn;
            Ok(Transition::Push(Box::new(game)))
        }))
        .item(MenuItem::new("Login as").on_activate(|_, _, _| {
            let mut game = Game::new()?;
            game.state = GameState::SettingAccount;
            Ok(Transition::Push(Box::new(game)))
        }))
        .item(MenuItem::new("create account").on_activate(|_, _, _| {
            let mut game = Game::new()?;
            game.state = GameState::Creating;
            Ok(Transition::Push(Box::new(game)))
        }))
        .item(MenuItem::new("Exit").on_activate(|_, _, _| Ok(Transition::Pop(1))))
        .with_music("sounds/music/main_menu.mp3")
        .build();
    let main_menu = Box::new(main_menu);
    states.push_state(main_menu, &mut ctx)?;
    let mut last_frame_time = Instant::now();
    event_loop.run(move |event, _, _| {
        if ctx.feed_event(&event) {
            // Todo: Better error handling
            if !states.on_update(&mut ctx).unwrap() {
                ctx.config.save().unwrap();
                std::process::exit(0);
            }
        }
        let elapsed = last_frame_time.elapsed();
        last_frame_time = Instant::now();

        // Sleep for the remaining frame time
        if let Some(remaining) = FRAME_TIME.checked_sub(elapsed) {
            thread::sleep(remaining);
        } else {
            // If the frame took longer than expected, consider skipping a frame
            // or handle it based on your game requirements
        }
    });
}

fn main() {
    if let Err(val) = run() {
        error!("{}", val);
    }
}
