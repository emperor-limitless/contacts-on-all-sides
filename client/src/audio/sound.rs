#![allow(unused)]
use crate::url_stream::UrlStream;
use once_cell::sync::Lazy;
use std::{collections::HashMap, fs};
use synthizer as syz;
static mut BUFFERS: Lazy<HashMap<String, syz::Buffer>> = Lazy::new(|| HashMap::new());
static mut STREAMS: Lazy<HashMap<String, Vec<u8>>> = Lazy::new(|| HashMap::new());

#[derive(Clone)]
pub struct Cache {
    pub ctx: syz::Context,
}
impl Cache {
    pub fn new(ctx: syz::Context) -> Self {
        Self {
            ctx,
        }
    }
    pub fn load_buffer(&self, file_path: &str) -> syz::Buffer {
        let encrypted = fs::read(file_path).unwrap();
        syz::Buffer::from_encoded_data(&encrypted).unwrap()
    }
    pub fn load_stream(&self, file_path: &str) -> syz::StreamHandle {
        unsafe {
            let encrypted = fs::read(file_path).unwrap();
            let filename = file_path.clone();
            if !STREAMS.contains_key(filename) {
                STREAMS.insert(String::from(filename.clone()), encrypted.clone());
            }
            syz::StreamHandle::from_vec(encrypted).unwrap()
        }
    }
    pub fn buffer(&self, file_path: &str) -> syz::Buffer {
        unsafe {
            if BUFFERS.contains_key(file_path) {
                BUFFERS.get(file_path).unwrap().clone()
            } else {
                let buf = self.load_buffer(file_path);
                BUFFERS.insert(String::from(file_path), buf);
                BUFFERS.get(file_path).unwrap().clone()
            }
        }
    }
    pub fn stream(&self, file_path: &str) -> syz::StreamHandle {
        unsafe {
            if STREAMS.contains_key(file_path) {
                syz::StreamHandle::from_vec(STREAMS.get(file_path).unwrap().clone()).unwrap()
            } else {
                self.load_stream(file_path)
            }
        }
    }
}
pub struct SoundManager {
    pub ctx: syz::Context,
    pub del_cfg: syz::DeleteBehaviorConfig,
    pub cache: Cache,
}
impl SoundManager {
    pub fn new(cache: Cache) -> Self {
        Self {
            ctx: cache.ctx.clone(),
            del_cfg: syz::DeleteBehaviorConfigBuilder::new()
                .linger(true)
                .linger_timeout(0.0)
                .build(),

            cache,
        }
    }
    pub fn set_position(&self, pos: (f64, f64, f64)) -> anyhow::Result<()> {
        self.ctx.position().set(pos)?;
        Ok(())
    }
    pub fn play(&self, filename: &str, looping: bool) -> Sound {
        let mut s = Sound::new(
            self.ctx.clone(),
            filename.clone(),
            self.cache.buffer(filename),
        );
        s.gen.buffer().set(&s.buf);
        s.src.add_generator(&s.gen);
        if looping {
            s.gen.looping().set(true);
        } else {
            s.src.config_delete_behavior(&self.del_cfg).unwrap();
            s.gen.config_delete_behavior(&self.del_cfg).unwrap();
        }
        s
    }
    pub fn play_wait(&self, filename: &str) -> Sound {
        let sy = self.play(filename, false);
        while sy.playing_s() {
            unsafe {}
        }
        sy
    }
    pub fn play_3d(&self, filename: &str, x: i32, y: i32, looping: bool) -> Sound3d {
        let mut s = Sound3d::new(
            self.ctx.clone(),
            filename.clone(),
            self.cache.buffer(filename),
        );
        s.set_position(x, y);
        s.gen.buffer().set(&s.buf);
        s.src.add_generator(&s.gen);
        if looping {
            s.gen.looping().set(true);
        }
        s.src.config_delete_behavior(&self.del_cfg).unwrap();
        s.gen.config_delete_behavior(&self.del_cfg).unwrap();
        s
    }

    pub fn play_s(&self, filename: &str, looping: bool, paused: bool) -> SoundStream {
        let mut s = SoundStream::new(
            self.ctx.clone(),
            filename.clone(),
            self.cache.stream(filename),
        );
        s.src.add_generator(&s.gen);
        if looping {
            s.gen.looping().set(true);
        }
        if paused {
            s.gen.pause();
        }
        s.src.config_delete_behavior(&self.del_cfg).unwrap();
        s.gen.config_delete_behavior(&self.del_cfg).unwrap();
        s
    }

    pub fn play_url(&self, url: &str, looping: bool, paused: bool) -> SoundStream {
        let sd = syz::CustomStreamDef::from_reader(UrlStream::new(url).unwrap());
        let sh = syz::StreamHandle::from_stream_def(sd).unwrap();
        let mut s = SoundStream::new(self.ctx.clone(), url.clone(), sh);
        s.src.add_generator(&s.gen);
        if looping {
            s.gen.looping().set(true);
        }
        if paused {
            s.gen.pause();
        }
        s
    }

    pub fn load(&self, filename: &str, looping: bool) -> Sound {
        let s = Sound::new(
            self.ctx.clone(),
            filename.clone(),
            self.cache.buffer(filename),
        );
        s.gen.buffer().set(&s.buf);
        s.src.add_generator(&s.gen);
        if looping {
            s.gen.looping().set(true);
        }
        s.gen.pause();
        s.src.config_delete_behavior(&self.del_cfg).unwrap();
        s.gen.config_delete_behavior(&self.del_cfg).unwrap();

        s
    }
    pub fn load3d(&self, filename: &str) -> Sound3d {
        let s = Sound3d::new(
            self.ctx.clone(),
            filename.clone(),
            self.cache.buffer(filename),
        );
        s.gen.buffer().set(&s.buf);
        s.src.add_generator(&s.gen);
        s.gen.pause();
        s.src.config_delete_behavior(&self.del_cfg).unwrap();
        s.gen.config_delete_behavior(&self.del_cfg).unwrap();

        s
    }
}
#[derive(Clone)]
pub struct Sound {
    pub ctx: syz::Context,
    pub src: syz::DirectSource,
    pub gen: syz::BufferGenerator,
    pub buf: syz::Buffer,
}
impl Sound {
    pub fn new(ctx: syz::Context, filename: &str, buf: syz::Buffer) -> Self {
        Self {
            gen: syz::BufferGenerator::new(&ctx).unwrap(),
            buf,
            src: syz::DirectSource::new(&ctx).unwrap(),
            ctx,
        }
    }
    pub fn play(&mut self, looping: bool) {
        self.gen.buffer().set(&self.buf);
        self.src.add_generator(&self.gen);
        if looping {
            self.gen.looping().set(true);
        }
    }

    pub fn playing(&self) -> bool {
        return (self.gen.playback_position().get().unwrap()
            <= self.buf.get_length_in_seconds().unwrap() - 1.700);
    }
    pub fn playing_s(&self) -> bool {
        return (self.gen.playback_position().get().unwrap()
            <= self.buf.get_length_in_seconds().unwrap() - 0.005);
    }

    pub fn volume(&mut self, vol: f64) {
        self.src.gain().set(vol);
    }
}
#[derive(Clone)]
pub struct Sound3d {
    pub ctx: syz::Context,
    pub src: syz::Source3D,
    pub gen: syz::BufferGenerator,
    pub buf: syz::Buffer,
    pub is_playing: bool,
}
impl Sound3d {
    pub fn new(ctx: syz::Context, filename: &str, buf: syz::Buffer) -> Self {
        Self {
            gen: syz::BufferGenerator::new(&ctx).unwrap(),
            buf,
            src: syz::Source3D::new(&ctx, syz::PannerStrategy::Hrtf, (0.0, 0.0, 0.0)).unwrap(),
            is_playing: false,
            ctx,
        }
    }
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.set_position_3d(x.into(), y.into(), 0.0).unwrap();
    }
    pub fn set_position_3d(&mut self, x: f64, y: f64, z: f64) -> anyhow::Result<()> {
        self.src.position().set((x, y, z))?;
        Ok(())
    }
    pub fn play(&mut self, looping: bool) {
        self.gen.buffer().set(&self.buf).unwrap();
        self.src.add_generator(&self.gen).unwrap();
        self.is_playing = true;
        if looping {
            self.gen.looping().set(true).unwrap();
        }
    }

    pub fn playing(&self) -> bool {
        return (self.gen.playback_position().get().unwrap()
            <= self.buf.get_length_in_seconds().unwrap() - 1.700);
    }
    pub fn playing_s(&self) -> bool {
        return (self.gen.playback_position().get().unwrap()
            <= self.buf.get_length_in_seconds().unwrap() - 0.005);
    }

    pub fn volume(&mut self, vol: f64) {
        self.src.gain().set(vol);
    }
}

#[derive(Clone)]
pub struct SoundStream {
    pub ctx: syz::Context,
    pub src: syz::DirectSource,
    pub gen: syz::StreamingGenerator,
}
impl SoundStream {
    pub fn new(ctx: syz::Context, filename: &str, sh: syz::StreamHandle) -> Self {
        Self {
            gen: syz::StreamingGenerator::from_stream_handle(&ctx, sh).unwrap(),
            src: syz::DirectSource::new(&ctx).unwrap(),
            ctx,
        }
    }
    pub fn play(&mut self, looping: bool) {
        self.src.add_generator(&self.gen);
        if looping {
            self.gen.looping().set(true);
        }
    }
    pub fn playing(&self) -> bool {
        return (self.gen.playback_position().get().unwrap() > 0.0);
    }

    pub fn volume(&mut self, vol: f64) {
        self.src.gain().set(vol);
    }
}
