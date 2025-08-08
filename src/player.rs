use eframe::egui;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use rfd::FileDialog;


pub struct AudioPlayer {
    audio_file: Option<PathBuf>,
    is_playing: bool,
    sink: Option<Arc<Mutex<Sink>>>,
    _stream: Option<OutputStream>,
    volume: f32,
    start_instant: Option<Instant>,
    duration: Option<Duration>,
    progress: f32, // 0.0 to 1.0
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self {
            audio_file: None,
            is_playing: false,
            sink: None,
            _stream: None,
            volume: 1.0,
            start_instant: None,
            duration: None,
            progress: 0.0,
        }
    }
}

impl eframe::App for AudioPlayer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use egui::{Align, Layout};

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(30.0);

                ui.heading("🎵 RustTune");
                ui.add_space(5.0);
                ui.label("A minimal audio player built with Rust + egui + rodio");
                ui.add_space(40.0);

                if ui
                    .add_sized([180.0, 40.0], egui::Button::new("📂 Load Audio File"))
                    .clicked()
                {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Audio", &["mp3", "wav"])
                        .pick_file()
                    {
                        self.audio_file = Some(path);
                        self.is_playing = false;
                        self.sink = None;
                        self._stream = None;
                        self.start_instant = None;
                        self.duration = None;
                        self.progress = 0.0;
                    }
                }

                ui.add_space(30.0);

                if let Some(path) = &self.audio_file {
                    let path = path.clone();

                    ui.group(|ui| {
                        ui.set_min_width(400.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.label(format!(
                                "🎵 Now Playing: {}",
                                path.file_name().unwrap().to_string_lossy()
                            ));
                            ui.add_space(10.0);
                        });
                    });

                    ui.add_space(25.0);

                    // Centered control buttons
                    ui.horizontal(|ui| {
                        let button_width = 120.0;
                        let spacing = 15.0;
                        let total_width = (button_width * 3.0) + (spacing * 2.0);
                        let center_offset = (ui.available_width() - total_width) / 2.0;

                        ui.add_space(center_offset.max(0.0));

                        // Play/Pause
                        let play_pause_text = if self.is_playing { "⏸ Pause" } else { "▶️ Play" };
                        if ui.add_sized([button_width, 50.0], egui::Button::new(play_pause_text)).clicked() {
                            if self.sink.is_none() {
                                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                                let file = File::open(&path).unwrap();
                                let source = Decoder::new(BufReader::new(file)).unwrap();

                                let total_duration = source.total_duration();
                                let target_secs = self.progress * total_duration.unwrap_or_default().as_secs_f32();

                                let skip_duration = Duration::from_secs_f32(target_secs);
                                let skipped_source = source.skip_duration(skip_duration);

                                let sink = Sink::try_new(&stream_handle).unwrap();
                                sink.append(skipped_source);
                                sink.set_volume(self.volume);
                                sink.play();

                                self.sink = Some(Arc::new(Mutex::new(sink)));
                                self._stream = Some(_stream);
                                self.is_playing = true;
                                self.start_instant = Some(Instant::now() - skip_duration);
                                self.duration = total_duration.or(Some(Duration::from_secs(180)));
                            } else if let Some(sink) = &self.sink {
                                let sink = sink.lock().unwrap();
                                if self.is_playing {
                                    sink.pause();
                                } else {
                                    sink.play();
                                }
                                self.is_playing = !self.is_playing;
                            }
                        }

                        ui.add_space(spacing);

                        // Replay
                        if ui.add_sized([button_width, 50.0], egui::Button::new("🔄 Replay")).clicked() {
                            self.progress = 0.0;
                            self.sink = None;
                            self._stream = None;

                            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                            let file = File::open(&path).unwrap();
                            let source = Decoder::new(BufReader::new(file)).unwrap();
                            let total_duration = source.total_duration();

                            let sink = Sink::try_new(&stream_handle).unwrap();
                            sink.append(source);
                            sink.set_volume(self.volume);
                            sink.play();

                            self.sink = Some(Arc::new(Mutex::new(sink)));
                            self._stream = Some(_stream);
                            self.is_playing = true;
                            self.start_instant = Some(Instant::now());
                            self.duration = total_duration.or(Some(Duration::from_secs(180)));
                        }

                        ui.add_space(spacing);

                        // Reset
                        if ui.add_sized([button_width, 50.0], egui::Button::new("🗑️ Reset")).clicked() {
                            self.sink = None;
                            self._stream = None;
                            self.audio_file = None;
                            self.is_playing = false;
                            self.start_instant = None;
                            self.duration = None;
                            self.progress = 0.0;
                        }
                    });
                    ui.add_space(30.0);

                    ui.group(|ui| {
                        ui.set_min_width(350.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(15.0);
                            ui.label("🔊 Volume");
                            ui.add_space(10.0);
                            let response = ui.add_sized(
                                [250.0, 20.0],
                                egui::Slider::new(&mut self.volume, 0.0..=1.0)
                                    .show_value(true)
                                    .text("Volume"),
                            );
                            if response.changed() {
                                if let Some(sink) = &self.sink {
                                    let sink = sink.lock().unwrap();
                                    sink.set_volume(self.volume);
                                }
                            }
                            ui.add_space(15.0);
                        });
                    });

                    ui.add_space(30.0);

                    if let Some(start) = self.start_instant {
                        if let Some(total) = self.duration {
                            let elapsed = if self.is_playing {
                                start.elapsed()
                            } else {
                                Duration::from_secs_f32(self.progress * total.as_secs_f32())
                            };

                            let new_progress = (elapsed.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0);

                            // Stop playback when song ends
                            if elapsed >= total && self.is_playing {
                                self.is_playing = false;
                                self.progress = 1.0;
                                self.sink = None;
                                self._stream = None;
                            } else {
                                self.progress = new_progress;
                            }

                            let display_elapsed = elapsed.min(total);

                            ui.group(|ui| {
                                ui.set_min_width(350.0);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(15.0);
                                    ui.label(format!(
                                        "⏱️ {:02}:{:02} / {:02}:{:02}",
                                        display_elapsed.as_secs() / 60,
                                        display_elapsed.as_secs() % 60,
                                        total.as_secs() / 60,
                                        total.as_secs() % 60
                                    ));

                                    ui.add_space(15.0);

                                    // Progress slider with seeking
                                    let mut new_progress = self.progress;
                                    let response = ui.add_sized(
                                        [320.0, 25.0],
                                        egui::Slider::new(&mut new_progress, 0.0..=1.0)
                                            .show_value(false)
                                            .text("Progress"),
                                    );

                                    // Seek on slider change
                                    if response.changed() {
                                        self.progress = new_progress;
                                        let skip_to = Duration::from_secs_f32(
                                            self.progress * total.as_secs_f32(),
                                        );

                                        self.sink = None;
                                        self._stream = None;

                                        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                                        let file = File::open(&path).unwrap();
                                        let source = Decoder::new(BufReader::new(file)).unwrap();
                                        let skipped = source.skip_duration(skip_to);

                                        let sink = Sink::try_new(&stream_handle).unwrap();
                                        sink.append(skipped);
                                        sink.set_volume(self.volume);

                                        if self.is_playing {
                                            sink.play();
                                        } else {
                                            sink.pause();
                                        }

                                        self.sink = Some(Arc::new(Mutex::new(sink)));
                                        self._stream = Some(_stream);
                                        self.start_instant = Some(Instant::now() - skip_to);
                                    }

                                    ui.add_space(15.0);
                                });
                            });
                        }
                    }
                } else {
                    ui.group(|ui| {
                        ui.set_min_width(300.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label("🧐 No audio file selected.");
                            ui.add_space(10.0);
                            ui.label("Click 'Load Audio File' to get started!");
                            ui.add_space(20.0);
                        });
                    });
                }

                ui.add_space(50.0);
            });
        });

        ctx.request_repaint_after(Duration::from_millis(500));
    }
}
