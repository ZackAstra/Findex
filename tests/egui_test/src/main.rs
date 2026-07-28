fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Test",
        options,
        Box::new(|_cc| Ok(Box::new(TestApp::default()))),
    )
}

struct TestApp;

impl Default for TestApp {
    fn default() -> Self { Self }
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello, Findex UI!");
        });
    }
}
