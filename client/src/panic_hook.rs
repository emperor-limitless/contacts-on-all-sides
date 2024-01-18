#![allow(unused)]
use backtrace::Backtrace;
use msgbox::IconType;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic::PanicInfo;

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
