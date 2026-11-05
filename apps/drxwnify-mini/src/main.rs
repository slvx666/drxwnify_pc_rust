use drxwnify_audio_engine::AudioEngine;
use drxwnify_core::{AppEdition, AppPaths, EventBus};
use drxwnify_library::LibraryManager;
use drxwnify_playback::Player;
use drxwnify_shared_models::*;
use eframe::egui;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MiniTheme {
    Night,
    Day,
    Gruvbox,
    Satanic,
    SakuraPixel,
    LinuxTerminal,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveScreen {
    Home,
    Search,
    Liked,
    Downloads,
    History,
    Friends,
}

struct DrxwnifyMiniApp {
    theme: MiniTheme,
    screen: ActiveScreen,
    search_query: String,
    player: Player,
    library: Arc<LibraryManager>,
    event_bus: EventBus,
    active_track: Option<Track>,
    is_playing: bool,
    position_ms: i64,
    duration_sec: i64,
    volume: f32,
    balance: f32,
    right_panel_visible: bool,
    left_collapsed: bool,
}

impl DrxwnifyMiniApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let paths = AppPaths::new(AppEdition::Mini);
        let _ = paths.ensure_directories();

        let event_bus = EventBus::default();
        let audio_engine = Arc::new(AudioEngine::new(event_bus.clone(), None));
        let http = drxwnify_network::HttpClient::new().unwrap_or_else(|_| drxwnify_network::HttpClient::default());
        let player = Player::new(audio_engine, event_bus.clone(), http);
        let library = Arc::new(LibraryManager::new(paths));

        let app = Self {
            theme: MiniTheme::Night,
            screen: ActiveScreen::Home,
            search_query: String::new(),
            player,
            library,
            event_bus,
            active_track: None,
            is_playing: false,
            position_ms: 0,
            duration_sec: 0,
            volume: 0.8,
            balance: 0.0,
            right_panel_visible: true,
            left_collapsed: false,
        };

        app.apply_theme(&cc.egui_ctx);
        app
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
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

impl eframe::App for DrxwnifyMiniApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Bar
        egui::TopBottomPanel::top("mini_top_bar").exact_height(46.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("DRXWNIFY MINI").strong());
                ui.separator();

                if ui.button("⏴").clicked() {
                    self.player.previous();
                }
                if ui.button("⏵").clicked() {
                    self.player.next();
                }

                ui.separator();
                ui.label("Search:");
                let search_resp = ui.text_edit_singleline(&mut self.search_query);
                if search_resp.lost_focus() && !self.search_query.trim().is_empty() {
                    self.screen = ActiveScreen::Search;
                }

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
                    if ui.button(if self.right_panel_visible { "Panel ⏵" } else { "Panel ⏴" }).clicked() {
                        self.right_panel_visible = !self.right_panel_visible;
                    }
                });
            });
        });

        // Bottom Player Bar
        egui::TopBottomPanel::bottom("mini_player_bar").exact_height(72.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let (rect, _response) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::hover());
                let cover_color = match self.theme {
                    MiniTheme::Satanic => egui::Color32::from_rgb(60, 20, 20),
                    MiniTheme::SakuraPixel => egui::Color32::from_rgb(60, 30, 50),
                    MiniTheme::LinuxTerminal => egui::Color32::from_rgb(20, 40, 40),
                    _ => egui::Color32::from_rgb(45, 50, 65),
                };
                ui.painter().rect_filled(rect, 4.0, cover_color);

                ui.vertical(|ui| {
                    let title = self.active_track.as_ref().map(|t| t.title.as_str()).unwrap_or("No Track Playing");
                    let artist = self.active_track.as_ref().map(|t| t.artist_names()).unwrap_or_else(|| "Idle".to_string());
                    ui.label(egui::RichText::new(title).strong());
                    ui.label(egui::RichText::new(format!("{} · 320 kbps [VK API]", artist)).small());
                });

                ui.separator();

                if ui.button(if self.is_playing { "⏸" } else { "▶" }).clicked() {
                    self.player.toggle_play_pause();
                    self.is_playing = !self.is_playing;
                }

                if ui.button("🔀").clicked() {
                    self.player.shuffle();
                }

                if ui.button("🔁").clicked() {
                    self.player.set_repeat_mode(RepeatMode::All);
                }

                ui.separator();

                let mut pos = (self.position_ms as f32 / 1000.0).max(0.0);
                let dur = self.duration_sec.max(1) as f32;
                if ui.add(egui::Slider::new(&mut pos, 0.0..=dur).show_value(false)).changed() {
                    self.player.seek((pos * 1000.0) as i64);
                }
                ui.label(format!("{}:{:02} / {}:{:02}", (pos as i64) / 60, (pos as i64) % 60, (dur as i64) / 60, (dur as i64) % 60));

                ui.separator();
                if ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Vol")).changed() {
                    self.player.set_volume(self.volume);
                }
            });
        });

        // Left Navigation
        let left_width = if self.left_collapsed { 52.0 } else { 220.0 };
        egui::SidePanel::left("mini_left_nav").exact_width(left_width).show(ctx, |ui| {
            if !self.left_collapsed {
                ui.heading("Library");
                ui.separator();
                if ui.selectable_label(self.screen == ActiveScreen::Home, "⌂  Home").clicked() {
                    self.screen = ActiveScreen::Home;
                }
                if ui.selectable_label(self.screen == ActiveScreen::Search, "🔍 Search").clicked() {
                    self.screen = ActiveScreen::Search;
                }
                if ui.selectable_label(self.screen == ActiveScreen::Liked, "♥  Liked Tracks").clicked() {
                    self.screen = ActiveScreen::Liked;
                }
                if ui.selectable_label(self.screen == ActiveScreen::Downloads, "⭳  Downloads").clicked() {
                    self.screen = ActiveScreen::Downloads;
                }
                if ui.selectable_label(self.screen == ActiveScreen::History, "◷  History").clicked() {
                    self.screen = ActiveScreen::History;
                }
                if ui.selectable_label(self.screen == ActiveScreen::Friends, "👥 Friends (Nostr)").clicked() {
                    self.screen = ActiveScreen::Friends;
                }

                ui.separator();
                ui.label(egui::RichText::new("PLAYLISTS").small());
                let _ = ui.button("Night Drive");
                let _ = ui.button("Post-Punk Gems");
            } else {
                if ui.button("⌂").clicked() { self.screen = ActiveScreen::Home; }
                if ui.button("🔍").clicked() { self.screen = ActiveScreen::Search; }
                if ui.button("♥").clicked() { self.screen = ActiveScreen::Liked; }
                if ui.button("⭳").clicked() { self.screen = ActiveScreen::Downloads; }
                if ui.button("◷").clicked() { self.screen = ActiveScreen::History; }
                if ui.button("👥").clicked() { self.screen = ActiveScreen::Friends; }
            }
        });

        // Right Info / Queue / Fastfetch Panel
        if self.right_panel_visible {
            egui::SidePanel::right("mini_right_panel").exact_width(260.0).show(ctx, |ui| {
                ui.heading("Queue & Info");
                ui.separator();

                if self.theme == MiniTheme::LinuxTerminal {
                    ui.monospace("drxwnify@mini-desktop");
                    ui.monospace("---------------------");
                    ui.monospace("edition  :: Mini Rust");
                    ui.monospace("theme    :: Linux TTY");
                    ui.monospace("codec    :: Opus / FLAC");
                    ui.monospace("backend  :: WASAPI CPAL");
                    ui.monospace("latency  :: < 12ms");
                    ui.monospace("ui       :: egui (0.0% idle)");
                } else {
                    ui.label("Playing Next:");
                    ui.label("1. Crystal Castles - Kept");
                    ui.label("2. Molchat Doma - Sudno");
                    ui.label("3. Pastel Ghost - Dark Beach");
                    ui.separator();
                    ui.label(egui::RichText::new("Lyrics:").strong());
                    ui.label("Everything is falling down...");
                    ui.label(egui::RichText::new("And the sound is pure again.").color(egui::Color32::from_rgb(0x5B, 0x8D, 0xEF)));
                    ui.label("Lost inside the echoes.");
                }
            });
        }

        // Central Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.screen {
                ActiveScreen::Home => {
                    ui.heading("Featured & Mixes");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("▶ Mix of the Day").clicked() {
                            let sample = Track::new("t1", "Resurrections", "Lena Raine", 218);
                            self.player.play_track(sample.clone());
                            self.active_track = Some(sample);
                            self.duration_sec = 218;
                            self.is_playing = true;
                        }
                        if ui.button("▶ Post-Punk Essentials").clicked() {
                            let sample = Track::new("t2", "Sudno", "Molchat Doma", 140);
                            self.player.play_track(sample.clone());
                            self.active_track = Some(sample);
                            self.duration_sec = 140;
                            self.is_playing = true;
                        }
                    });
                    ui.add_space(12.0);
                    ui.heading("Your Liked Library");
                    let liked = self.library.get_liked();
                    if liked.is_empty() {
                        ui.label("Your collection is ready. Add tracks from search or import Android backup!");
                    } else {
                        for t in liked.iter().take(15) {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&t.title).strong());
                                ui.label(&t.artist_names());
                            });
                        }
                    }
                }
                ActiveScreen::Search => {
                    ui.heading(format!("Search Results: '{}'", self.search_query));
                    ui.separator();
                    for i in 1..=15 {
                        ui.horizontal(|ui| {
                            ui.monospace(format!("{:2}.", i));
                            let title = format!("Result #{} for {}", i, self.search_query);
                            ui.label(egui::RichText::new(&title).strong());
                            ui.label("Various Artists");
                            if ui.button("▶").clicked() {
                                let track = Track::new(format!("s_{}", i), title, "Various Artists", 200);
                                self.player.play_track(track.clone());
                                self.active_track = Some(track);
                                self.duration_sec = 200;
                                self.is_playing = true;
                            }
                        });
                    }
                }
                ActiveScreen::Liked => {
                    ui.heading("♥ Liked Tracks");
                    ui.separator();
                    let liked = self.library.get_liked();
                    ui.label(format!("Total liked: {} tracks", liked.len()));
                }
                ActiveScreen::Downloads => {
                    ui.heading("⭳ Downloads");
                    ui.separator();
                    ui.label("Saved audio files in %USERPROFILE%\\Music\\Drxwnify");
                }
                ActiveScreen::History => {
                    ui.heading("◷ Listening History");
                    ui.separator();
                    let hist = self.library.get_history();
                    ui.label(format!("History entries: {}", hist.len()));
                }
                ActiveScreen::Friends => {
                    ui.heading("👥 Friends & Nostr ChatHub");
                    ui.separator();
                    ui.label("Nostr Relay status: 5 public relays active");
                    ui.label("Kind 4 Direct Messages ready");
                }
            }
        });

        if self.is_playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([880.0, 580.0])
            .with_title("Drxwnify Mini — High-Efficiency Audio Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "Drxwnify Mini",
        native_options,
        Box::new(|cc| Ok(Box::new(DrxwnifyMiniApp::new(cc)))),
    )
}
