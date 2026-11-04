use eframe::egui;
use sysinfo::{Pid, System};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MiniTheme {
    Night,
    Day,
    Gruvbox,
    Satanic,
    SakuraPixel,
    LinuxTerminal,
}

struct TrackItem {
    id: usize,
    title: String,
    artist: String,
    duration: String,
}

struct MiniBenchmarkApp {
    theme: MiniTheme,
    search_query: String,
    volume: f32,
    progress: f32,
    is_playing: bool,
    tracks: Vec<TrackItem>,
    start_time: Instant,
    initial_ram_mb: f64,
    current_ram_mb: f64,
    system: System,
    pid: Pid,
    frames_rendered: u64,
}

impl MiniBenchmarkApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Custom fonts or styling
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = Pid::from_u32(std::process::id());
        let initial_ram = sys.process(pid).map(|p| p.memory() as f64 / (1024.0 * 1024.0)).unwrap_or(0.0);

        let mut tracks = Vec::with_capacity(2000);
        for i in 1..=2000 {
            tracks.push(TrackItem {
                id: i,
                title: format!("Test Track #{} - Crystal Memories", i),
                artist: format!("Artist {}", (i % 25) + 1),
                duration: format!("{}:{:02}", 2 + (i % 3), (i * 17) % 60),
            });
        }

        let app = Self {
            theme: MiniTheme::Night,
            search_query: String::new(),
            volume: 0.8,
            progress: 0.35,
            is_playing: false,
            tracks,
            start_time: Instant::now(),
            initial_ram_mb: initial_ram,
            current_ram_mb: initial_ram,
            system: sys,
            pid,
            frames_rendered: 0,
        };

        app.apply_theme(&cc.egui_ctx);
        app
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mut visuals = match self.theme {
            MiniTheme::Night => {
                let mut v = egui::Visuals::dark();
                v.panel_fill = egui::Color32::from_rgb(0x1E, 0x20, 0x26);
                v.window_fill = egui::Color32::from_rgb(0x12, 0x13, 0x16);
                v.extreme_bg_color = egui::Color32::from_rgb(0x18, 0x1A, 0x1F);
                v.override_text_color = Some(egui::Color32::from_rgb(0xE6, 0xE8, 0xEE));
                v
            }
            MiniTheme::Day => {
                let mut v = egui::Visuals::light();
                v.panel_fill = egui::Color32::from_rgb(0xDF, 0xD9, 0xCD);
                v.window_fill = egui::Color32::from_rgb(0xEF, 0xEC, 0xE6);
                v.extreme_bg_color = egui::Color32::from_rgb(0xE6, 0xE2, 0xD8);
                v.override_text_color = Some(egui::Color32::from_rgb(0x26, 0x24, 0x20));
                v
            }
            MiniTheme::Gruvbox => {
                let mut v = egui::Visuals::dark();
                v.panel_fill = egui::Color32::from_rgb(0x32, 0x30, 0x2F);
                v.window_fill = egui::Color32::from_rgb(0x1D, 0x20, 0x21);
                v.extreme_bg_color = egui::Color32::from_rgb(0x28, 0x28, 0x28);
                v.override_text_color = Some(egui::Color32::from_rgb(0xEB, 0xDB, 0xB2));
                v
            }
            MiniTheme::Satanic => {
                let mut v = egui::Visuals::dark();
                v.panel_fill = egui::Color32::from_rgb(0x1A, 0x14, 0x14);
                v.window_fill = egui::Color32::from_rgb(0x0C, 0x0A, 0x0A);
                v.extreme_bg_color = egui::Color32::from_rgb(0x14, 0x10, 0x10);
                v.override_text_color = Some(egui::Color32::from_rgb(0xE8, 0xDF, 0xD8));
                v
            }
            MiniTheme::SakuraPixel => {
                let mut v = egui::Visuals::dark();
                v.panel_fill = egui::Color32::from_rgb(0x27, 0x1F, 0x2E);
                v.window_fill = egui::Color32::from_rgb(0x14, 0x10, 0x16);
                v.extreme_bg_color = egui::Color32::from_rgb(0x1D, 0x17, 0x22);
                v.override_text_color = Some(egui::Color32::from_rgb(0xF5, 0xE8, 0xEE));
                v
            }
            MiniTheme::LinuxTerminal => {
                let mut v = egui::Visuals::dark();
                v.panel_fill = egui::Color32::from_rgb(0x1C, 0x24, 0x29);
                v.window_fill = egui::Color32::from_rgb(0x0F, 0x14, 0x17);
                v.extreme_bg_color = egui::Color32::from_rgb(0x16, 0x1C, 0x20);
                v.override_text_color = Some(egui::Color32::from_rgb(0xC5, 0xD1, 0xD9));
                v
            }
        };
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
        visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
        visuals.widgets.active.rounding = egui::Rounding::same(4.0);
        ctx.set_visuals(visuals);
    }
}

impl eframe::App for MiniBenchmarkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frames_rendered += 1;

        if self.frames_rendered % 30 == 0 {
            self.system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[self.pid]));
            if let Some(p) = self.system.process(self.pid) {
                self.current_ram_mb = p.memory() as f64 / (1024.0 * 1024.0);
            }
        }

        // Top Bar
        egui::TopBottomPanel::top("top_bar").exact_height(46.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("DRXWNIFY MINI").strong());
                ui.separator();
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);

                ui.separator();
                let prev_theme = self.theme;
                egui::ComboBox::from_label("Theme")
                    .selected_text(format!("{:?}", self.theme))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.theme, MiniTheme::Night, "Night");
                        ui.selectable_value(&mut self.theme, MiniTheme::Day, "Day");
                        ui.selectable_value(&mut self.theme, MiniTheme::Gruvbox, "Gruvbox");
                        ui.selectable_value(&mut self.theme, MiniTheme::Satanic, "Satanic");
                        ui.selectable_value(&mut self.theme, MiniTheme::SakuraPixel, "Sakura Pixel");
                        ui.selectable_value(&mut self.theme, MiniTheme::LinuxTerminal, "Linux Terminal");
                    });
                if self.theme != prev_theme {
                    self.apply_theme(ctx);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("RAM: {:.1} MB | Frames: {}", self.current_ram_mb, self.frames_rendered));
                });
            });
        });

        // Bottom Player Bar
        egui::TopBottomPanel::bottom("player_bar").exact_height(72.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Cover preview
                let (rect, _response) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, egui::Color32::from_rgb(60, 60, 80));

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Current Playing Track").strong());
                    ui.label(egui::RichText::new("Featured Artist · 320 kbps [VK API]").small());
                });

                ui.separator();

                if ui.button(if self.is_playing { "⏸ Pause" } else { "▶ Play" }).clicked() {
                    self.is_playing = !self.is_playing;
                }

                ui.add(egui::Slider::new(&mut self.progress, 0.0..=1.0).show_value(false).text("Progress"));
                ui.label("1:14 / 3:45");

                ui.separator();
                ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Vol"));
            });
        });

        // Left Nav
        egui::SidePanel::left("left_nav").exact_width(220.0).show(ctx, |ui| {
            ui.heading("Library");
            ui.separator();
            let _ = ui.selectable_label(true, "⌂  Home");
            let _ = ui.selectable_label(false, "🔍 Search");
            let _ = ui.selectable_label(false, "♥  Liked Tracks (1,420)");
            let _ = ui.selectable_label(false, "⭳  Downloads (85)");
            let _ = ui.selectable_label(false, "◷  History");
            let _ = ui.selectable_label(false, "👥 Friends (Nostr)");
            ui.separator();
            ui.label(egui::RichText::new("PLAYLISTS").small());
            let _ = ui.button("Night Drive");
            let _ = ui.button("Post-Punk Gems");
            let _ = ui.button("Vaporwave Dreams");
        });

        // Right Info Panel
        egui::SidePanel::right("right_panel").exact_width(260.0).show(ctx, |ui| {
            ui.heading("Queue & Info");
            ui.separator();
            if self.theme == MiniTheme::LinuxTerminal {
                ui.monospace("drxwnify@mini-pc");
                ui.monospace("----------------");
                ui.monospace("codec   :: Opus 160k");
                ui.monospace("engine  :: WASAPI");
                ui.monospace("latency :: 12ms");
                ui.monospace("tracks  :: 2,000");
            } else {
                ui.label("Next in Queue:");
                ui.label("1. Crystal Castles - Kept");
                ui.label("2. Molchat Doma - Sudno");
                ui.label("3. Pastel Ghost - Dark Beach");
            }
        });

        // Central Area: 2000 virtualized tracks
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Collection ({} tracks)", self.tracks.len()));
            ui.separator();

            let row_height = 28.0;
            let num_rows = self.tracks.len();
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show_rows(ui, row_height, num_rows, |ui, row_range| {
                for i in row_range {
                    let track = &self.tracks[i];
                    ui.horizontal(|ui| {
                        ui.monospace(format!("{:3}.", track.id));
                        ui.label(egui::RichText::new(&track.title).strong());
                        ui.label(&track.artist);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(&track.duration);
                        });
                    });
                }
            });
        });

        // If not playing, don't continuously repaint (0% CPU)
        if self.is_playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([880.0, 580.0])
            .with_title("Drxwnify Mini — Benchmark Prototype"),
        ..Default::default()
    };

    println!("Starting Drxwnify Mini Benchmark Prototype...");
    let start = Instant::now();
    eframe::run_native(
        "Drxwnify Mini",
        native_options,
        Box::new(move |cc| {
            println!("Initialization took: {:?}", start.elapsed());
            Ok(Box::new(MiniBenchmarkApp::new(cc)))
        }),
    )
}
