//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]
#![allow(non_upper_case_globals)]
use anyhow;
use backtrace::Backtrace;
use msgbox::IconType;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic;
use std::panic::PanicInfo;
use synthizer;
mod context;
mod dm;
mod input;
mod menu;
mod sound;
mod state_manager;

use winit::event_loop::EventLoop;

use crate::{context::GameContext, input::Input, menu::Menu, state_manager::StateManager};

pub fn panic_handler(panic_info: &PanicInfo) {
    let backtrace = Backtrace::new();
    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("<unnamed>");
    msgbox::create(
        "Error",
        &format!("thread '{}' {}", thread_name, panic_info),
        IconType::Error,
    );
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open("panics.log")
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Warning: Could not open panics.log: {}", e);
            eprintln!("thread '{}' {} \n {:?}", thread_name, panic_info, backtrace);
            return;
        }
    };
    if let Err(e) = writeln!(
        file,
        "thread '{}' {} \n {:?}",
        thread_name, panic_info, backtrace
    ) {
        eprintln!("Warning: Could not write to panics.log: {}", e);
        eprintln!("thread '{}' {} \n {:?}", thread_name, panic_info, backtrace);
    }
}

fn main() -> anyhow::Result<()> {
    panic::set_hook(Box::new(panic_handler));
    let init_guard = synthizer::initialize()?;
    let event_loop = EventLoop::<()>::new();
    let mut ctx = GameContext::new(&event_loop)?;
    let mut states = StateManager::<GameContext>::new(4);
    let main_menu = Box::new(
        Menu::new(String::from("Main"), ["Lets type", "Exit"]).set_callback(Box::new(
            |_ctx, name| -> anyhow::Result<state_manager::Transition<GameContext>> {
                if name == "Exit" {
                    return Ok(state_manager::Transition::Pop(1));
                } else if name == "Lets type" {
                    let i = Box::new(
                        Input::new()
                            .set_title("Enter you're text".to_string())
                            .set_callback(Box::new(|ctx, result| {
                                if result != "" {
                                    ctx.speaker.speak(result, true)?;
                                } else {
                                    return Ok(state_manager::Transition::Pop(1));
                                }
                                Ok(state_manager::Transition::None)
                            })),
                    );
                    return Ok(state_manager::Transition::Push(i));
                }
                Ok(state_manager::Transition::None)
            },
        )),
    );
    states.push_state(main_menu, &mut ctx)?;
    event_loop.run(move |event, _, _| {
        if ctx.feed_event(&event) {
            // Todo: Better error handling
            if !states.on_update(&mut ctx).unwrap() {
                std::process::exit(0);
            }
        }
    });
}
