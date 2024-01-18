use crate::{
    audio::sound::{Cache, SoundManager, SoundStream},
    client::Client,
    config::Config as Cfg,
};
use anyhow::Context;
use enet::Enet;
use rand::rngs::ThreadRng;
use std::env::consts;
use synthizer as syz;
use tts::Tts;
use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

use log::{info, LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

const WINDOW_TITLE: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

pub struct GameContext {
    enet: Enet,
    pub win: Window,
    pub input: WinitInputHelper,
    pub speaker: Tts,
    pub sound: SoundManager,
    pub stream: Option<SoundStream>,
    pub config: Cfg,
    pub client: Option<Client>,
    pub rng: ThreadRng,
    _init_guard: syz::InitializationGuard,
    _log_handle: log4rs::Handle,
}

impl GameContext {
    pub fn new(event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
        let _log_handle = init_log()?;
        info!(
            "{} Version: {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        info!(
            "Target OS:  {} {}",
            os_version::detect()?.to_string(),
            consts::ARCH,
        );
        let win = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(640, 480))
            .with_resizable(false)
            .build(&event_loop)
            .context("Could not create window")?;
        let init_config = syz::LibraryConfig::new();
        let _init_guard = init_config.initialize()?;
        let (major, minor, patch) = syz::get_version();
        info!("Synthizer version {}.{}.{}", major, minor, patch);
        let cache = Cache::new(syz::Context::new()?);
        Ok(Self {
            enet: Enet::new()?,
            win,
            _log_handle,
            stream: None,
            input: WinitInputHelper::new(),
            speaker: Tts::default().context("Could not initialize TTS engine")?,
            client: None,
            rng: rand::thread_rng(),
            _init_guard,
            config: Cfg::new(),
            sound: SoundManager::new(cache),
        })
    }
    pub fn connect(&mut self) -> anyhow::Result<()> {
        self.client = Some(Client::new("localhost", 18832, &mut self.enet)?);
        Ok(())
    }

    #[inline]
    pub fn feed_event(&mut self, e: &Event<()>) -> bool {
        self.input.update(e)
    }
}

fn init_log() -> anyhow::Result<log4rs::Handle> {
    let level = log::LevelFilter::Info;
    let file_path = format!(
        "{}/{}.log",
        dirs::document_dir().unwrap().to_string_lossy(),
        env!("CARGO_PKG_NAME")
    );
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path)?;
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )?;
    Ok(log4rs::init_config(config)?)
}
