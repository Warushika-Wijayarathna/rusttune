mod player;

use eframe::egui;
use player::AudioPlayer;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "RustTune Audio Player",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark()); // 💡 enable dark mode
            Box::new(AudioPlayer::default())
        })
    )
}
