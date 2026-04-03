use eframe::egui;
use enigo::{Button, Direction, Enigo, Mouse, Settings};
use evdev::Device;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::{thread, time::Duration};

fn main() -> eframe::Result<()> {
    let is_running = Arc::new(AtomicBool::new(false));
    let is_pressing = Arc::new(AtomicBool::new(false));
    let target_cps = Arc::new(AtomicU64::new(15));

    let is_run_clone = Arc::clone(&is_running);
    let is_press_clone = Arc::clone(&is_pressing);
    let target_cps_clone = Arc::clone(&target_cps);

    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).expect("Failed to init Enigo");
        let is_p = Arc::clone(&is_press_clone);

        thread::spawn(move || {
            let mut device = Device::open("/dev/input/event7").expect("Failed to open event7");
            loop {
                if let Ok(events) = device.fetch_events() {
                    for event in events {
                        if event.code() == 272 {
                            is_p.store(event.value() == 1, Ordering::SeqCst);
                        }
                    }
                }
            }
        });

        loop {
            if is_run_clone.load(Ordering::SeqCst) && is_press_clone.load(Ordering::SeqCst) {
                let _ = enigo.button(Button::Left, Direction::Click);
                let current_cps = target_cps_clone.load(Ordering::SeqCst);
                let base_delay = 1000 / current_cps;
                let jitter = rand::random_range(5..25);
                let final_delay = if base_delay > jitter { base_delay - jitter } else { base_delay };
                thread::sleep(Duration::from_millis(final_delay));
            }
            thread::sleep(Duration::from_millis(5));
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([350.0, 320.0])
        .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Erro Clicker Pro 🛠️",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ClickerApp {
                is_running,
                target_cps,
            }))
        }),
    )
}

struct ClickerApp {
    is_running: Arc<AtomicBool>,
    target_cps: Arc<AtomicU64>,
}

impl eframe::App for ClickerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("🖱️ Error Clicker v1.0");
                ui.separator();
            });

            ui.add_space(15.0);

            let running = self.is_running.load(Ordering::SeqCst);
            let btn_text = if running { "STOP ENGINE" } else { "START ENGINE" };
            let btn_color = if running { egui::Color32::from_rgb(200, 0, 0) } else { egui::Color32::from_rgb(0, 150, 0) };

            if ui.add_sized([ui.available_width(), 40.0], egui::Button::new(btn_text).fill(btn_color)).clicked() {
                self.is_running.store(!running, Ordering::SeqCst);
            }

            ui.add_space(20.0);

            ui.group(|ui| {
                ui.label("Target Speed (CPS):");
                let mut val = self.target_cps.load(Ordering::SeqCst) as f32;
                if ui.add(egui::Slider::new(&mut val, 1.0..=30.0).step_by(1.0)).changed() {
                    self.target_cps.store(val as u64, Ordering::SeqCst);
                }
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.label("System Status:");
                if self.is_running.load(Ordering::SeqCst) {
                    ui.label(egui::RichText::new("READY ✅").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("IDLE 💤").color(egui::Color32::YELLOW));
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label("Wayland Protocol: Native (Enigo)");
                ui.label("Created By error404 ⚡");
            });
        });
    }

}
