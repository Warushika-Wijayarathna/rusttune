use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;

pub struct AudioPlayer {
    audio_file: Option<PathBuf>,
    is_playing: bool,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self {
            audio_file: None,
            is_playing: false,
        }
    }
}

impl eframe::App for AudioPlayer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RustTune Audio Player");
            if ui.button("Load Audio Files").clicked() {
                if let Some(path) = FileDialog::new().add_filter("Audio", &["mp3", "wav"]).pick_file() {
                    self.audio_file = Some(path);
                    self.is_playing = false;
                }
            }

            if let Some(path) = &self.audio_file {
                ui.label(format!("Selected file: {}", path.display()));
                if ui.button(if self.is_playing { "Pause" } else { "Play" }).clicked() {
                    self.is_playing = !self.is_playing;
                    // Here you would add the logic to play or pause the audio file
                    // For example, using a library like rodio or cpal
                }
            } else {
                ui.label("No audio file selected.");
            }
        });
    }
}
