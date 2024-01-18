    pub fn update_prepare(&mut self, text: &str) {
        let mut url = "";
        if cfg!(linux) {
            url = "https://dragon-creations.com/lom/legend_of_mortiea_linux.zip";
        } else if cfg!(macos) {
            url = "https://dragon-creations.com/lom/legend_of_mortiea_mac.zip";
        } else if cfg!(windows) {
            url = "https://dragon-creations.com/lom/legend_of_mortiea.zip";
        }

        let reader = reqwest::blocking::get(url).unwrap();
        self.tran_size = reader.content_length().unwrap();
        let writer = std::fs::File::create("legend_of_mortiea.zip").unwrap();
        self.transfer = Some(SizedTransfer::new(reader, writer, self.tran_size));
        self.tts.speak(
            "Downloading update, Press Space for percentage: Pressthe numbers 1 to 4 for other download information.",
            true,
        );
        self.update_callback();
    }
    pub fn update_callback(&mut self) {
        loop {
            self.update();
            if let Some(trans) = self.transfer.as_ref() {
                if trans.is_complete() {
                    self.update_done();
                    return;
                }
                let vr = (trans.fraction_transferred() * 100.0) as usize;
                if vr > self.vr {
                    let mut buf = make_buf(
                        ((MAX_FREQ - MIN_FREQ) * trans.fraction_transferred() as f32) + MIN_FREQ,
                        0.5,
                    );
                    self.buffer = syz::Buffer::from_float_array(SAMPLE_RATE, 1, &buf).unwrap();
                    self.gen = syz::BufferGenerator::new(&self.mm.ctx.clone()).unwrap();
                    self.src = syz::DirectSource::new(&self.mm.ctx.clone()).unwrap();
                    self.gen.buffer().set(&self.buffer);
                    self.src.add_generator(&self.gen);
                    self.src.config_delete_behavior(&self.mm.del_cfg).unwrap();
                    self.gen.config_delete_behavior(&self.mm.del_cfg).unwrap();
                    self.vr = vr;
                }
                if key_pressed(Scancode::Num1) {
                    self.tts.speak(
                        format!("{}MB Downloaded.", trans.transferred() / 1024 / 1024),
                        true,
                    );
                } else if key_pressed(Scancode::Num2) {
                    self.tts.speak(
                        format!(
                            "{}MB remaining.",
                            ((self.tran_size - trans.transferred()) / 1024 / 1024) as usize
                        ),
                        true,
                    );
                } else if key_pressed(Scancode::Num3) {
                    self.tts
                        .speak(&format!("{}MB/S.", trans.speed() / 1024 / 1024), true);
                } else if key_pressed(Scancode::Num4) {
                    if let Some(eta) = trans.eta() {
                        self.tts.speak(
                            &format!(
                                "The download is going to complete in approximately {:?}",
                                eta
                            ),
                            true,
                        );
                    } else {
                        self.tts.speak("Unknown download time remaining.", true);
                    }
                } else if key_pressed(Scancode::Space) {
                    self.tts.speak(
                        &format!("{}%", (trans.fraction_transferred() * 100.0) as usize),
                        true,
                    );
                }
            }
        }
    }
