use std::{collections::HashMap, sync::{Arc, Mutex}, time::Duration};
use egui::{Align, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Frame, Layout, Margin, RichText, Rounding, Stroke, Theme, Vec2, Visuals};

use chrono::{DateTime, Utc};
use rusty_trading_model::structs::{TimeSeries};

use crate::{create_new_stock_window, Stock};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: Arc<Mutex<f32>>,

    candle_toggle: bool,
    line_toggle: bool,

    // last time the data is updated
    last_update: DateTime<Utc>,

    stock: String,
    qty: String,
    price: String,
    // TODO: Refactor this with DashMap?
    stocks_map: Arc<Mutex<HashMap<String, Arc<Mutex<Stock>>>>>,
    
    // New UI state fields
    #[serde(skip)]
    connection_status: String,
    #[serde(skip)]
    total_portfolio_value: f64,
    #[serde(skip)]
    daily_pnl: f64,
    #[serde(skip)]
    show_help: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let app = Self {
            label: "Rusty Trading Platform".to_owned(),
            candle_toggle: true,
            line_toggle: false,
            value: Arc::new(Mutex::new(2.7)),
            last_update: Utc::now(),
            stock: String::new(),
            qty: String::new(),
            price: String::new(),
            stocks_map: Arc::new(Mutex::new(HashMap::new())),
            connection_status: "Connected".to_owned(),
            total_portfolio_value: 0.0,
            daily_pnl: 0.0,
            show_help: false,
        };
        app
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up custom dark theme for trading
        Self::setup_custom_style(&cc.egui_ctx);
        Self::setup_custom_fonts(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn setup_custom_style(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        
        // Trading-specific color scheme
        // visuals.window_fill = Color32::from_rgb(20, 25, 30);
        // visuals.panel_fill = Color32::from_rgb(25, 30, 35);
        // visuals.faint_bg_color = Color32::from_rgb(30, 35, 40);
        // visuals.extreme_bg_color = Color32::from_rgb(15, 20, 25);
        
        // Button colors
        // visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 45, 50);
        // visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 55, 60);
        // visuals.widgets.active.bg_fill = Color32::from_rgb(60, 65, 70);
        
        // Trading colors
        visuals.selection.bg_fill = Color32::from_rgb(0, 100, 0); // Green for profits
        visuals.hyperlink_color = Color32::from_rgb(100, 150, 255);
        
        // Window styling
        visuals.window_corner_radius = CornerRadius::default().at_least(8);
        visuals.menu_corner_radius = CornerRadius::default().at_least(6);
        
        ctx.set_theme(Theme::Dark);
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "pt-root-light".into(),
            Arc::new(FontData::from_static(include_bytes!("../assets/fonts/pt-root-ui_light.ttf"))),
        );
        fonts.font_data.insert(
            "pt-root-regular".into(),
            Arc::new(FontData::from_static(include_bytes!("../assets/fonts/pt-root-ui_regular.ttf"))),
        );
        fonts.font_data.insert(
            "pt-root-medium".into(),
            Arc::new(FontData::from_static(include_bytes!("../assets/fonts/pt-root-ui_medium.ttf"))),
        );
        fonts.font_data.insert(
            "pt-root-bold".into(),
            Arc::new(FontData::from_static(include_bytes!("../assets/fonts/pt-root-ui_bold.ttf"))),
        );

        if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
            proportional.insert(0, "pt-root-regular".into());
            proportional.insert(1, "pt-root-medium".into());
            proportional.insert(2, "pt-root-light".into());
            proportional.insert(3, "pt-root-bold".into());
        }

        if let Some(monospace) = fonts.families.get_mut(&FontFamily::Monospace) {
            monospace.insert(0, "pt-root-medium".into());
            monospace.insert(1, "pt-root-bold".into());
        }

        ctx.set_fonts(fonts);
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx, frame);

        // Update data periodically
        let now = Utc::now();
        if self.last_update + Duration::from_secs(1) <= now {
            self.update_market_data(ctx);
            self.last_update = now;
        }

        // Top menu bar with enhanced styling
        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        // App title
                        ui.label(RichText::new("Rusty Trading").size(23.0).color(Color32::from_rgb(255, 165, 0)));
                        ui.separator();
                        
                        // File menu
                        let is_web = cfg!(target_arch = "wasm32");
                        if !is_web {
                            ui.menu_button("File", |ui| {
                                if ui.button("📊 New Watchlist").clicked() {
                                    // TODO: Implement watchlist functionality
                                }
                                if ui.button("💾 Save Layout").clicked() {
                                    // TODO: Implement layout saving
                                }
                                ui.separator();
                                if ui.button("❌ Quit").clicked() {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            });
                        }
                        
                        // View menu
                        ui.menu_button("View", |ui| {
                            ui.checkbox(&mut self.show_help, "📖 Show Help");
                        });
                        
                        ui.add_space(26.0);
                    });
                    
                    // Right side - status and theme toggle
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        egui::widgets::global_theme_preference_buttons(ui);
                        ui.separator();
                        
                        // Connection status
                        let status_color = if self.connection_status == "Connected" {
                            Color32::from_rgb(0, 255, 0)
                        } else {
                            Color32::from_rgb(255, 0, 0)
                        };
                        ui.label(RichText::new(format!("🔗 {}", self.connection_status)).color(status_color));
                        
                        ui.separator();
                        
                        // Portfolio summary
                        ui.label(RichText::new(format!("💰 ${:.2}", self.total_portfolio_value)));
                        
                        let pnl_color = if self.daily_pnl >= 0.0 {
                            Color32::from_rgb(0, 255, 0)
                        } else {
                            Color32::from_rgb(255, 0, 0)
                        };
                        ui.label(RichText::new(format!("📈 {:.2}%", self.daily_pnl)).color(pnl_color));
                    });
                });
            });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("⏰ {}", now.format("%H:%M:%S UTC"))).size(18.0));
                    ui.separator();
                    ui.label(RichText::new(format!("📊 {} Active Positions", self.stocks_map.lock().unwrap().len())).size(18.0));
                    
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("❓").on_hover_text("Show keyboard shortcuts").clicked() {
                            self.show_help = !self.show_help;
                        }
                    });
                });
            });

        // Left side panel for trading controls
        egui::SidePanel::left("trading_panel")
            .min_width(250.0)
            .show(ctx, |ui| {
                self.show_trading_panel(ui);
            });

        // Central area for charts
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.show_charts_area(ui, ctx);
            });

        // Help window
        if self.show_help {
            self.show_help_window(ctx);
        }
    }
}

impl TemplateApp {
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F1) {
                self.show_help = !self.show_help;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Q) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                // TODO: New stock picker shortcut
            }
        });
    }

    fn update_market_data(&mut self, ctx: &egui::Context) {
        let ctx_clone = ctx.clone();
        let mut map = self.stocks_map.lock().unwrap();
        for (key, val) in map.iter_mut() {
            let request_template = ehttp::Request::get(format!("http://127.0.0.1:3000/stock?stock={key}"));
            log::info!("now is {:?}", Utc::now());
            log::info!("calling get_stock api and repaint graph");
            let val_clone = Arc::clone(&val);
            ehttp::fetch(request_template, move |result: ehttp::Result<ehttp::Response>| {
                let mut time_series: TimeSeries = serde_json::from_slice(&result.unwrap().bytes).unwrap();
                log::info!("time series size {}", time_series.data().len());
                val_clone.lock().unwrap().set_time_series(time_series);
            });
        }
        ctx_clone.request_repaint();
    }

    fn show_trading_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("📈 Trading Panel").size(16.0));
        ui.separator();
        
        // Stock picker section
        ui.group(|ui| {
            ui.label(RichText::new("🔍 Add Stock").size(14.0).strong());
            ui.horizontal(|ui| {
                ui.label("Symbol:");
                ui.text_edit_singleline(&mut self.stock);
            });
            
            ui.horizontal(|ui| {
                if ui.button(RichText::new("➕ Add to Watchlist").size(12.0)).clicked() {
                    if !self.stock.is_empty() {
                        self.stocks_map.lock().unwrap().insert(
                            self.stock.clone(),
                            Arc::new(Mutex::new(Stock::default(&self.stock)))
                        );
                        self.stock.clear();
                    }
                }
            });
        });
        
        ui.add_space(10.0);
        
        // Quick trade section
        ui.group(|ui| {
            ui.label(RichText::new("⚡ Quick Trade").size(14.0).strong());
            ui.horizontal(|ui| {
                ui.label("Qty:");
                ui.text_edit_singleline(&mut self.qty);
            });
            ui.horizontal(|ui| {
                ui.label("Price:");
                ui.text_edit_singleline(&mut self.price);
            });
            
            ui.horizontal(|ui| {
                let buy_button = ui.add(egui::Button::new(RichText::new("BUY").color(Color32::WHITE))
                    .fill(Color32::from_rgb(0, 150, 0)));
                if buy_button.clicked() {
                    // TODO: Implement quick buy
                }
                
                let sell_button = ui.add(egui::Button::new(RichText::new("SELL").color(Color32::WHITE))
                    .fill(Color32::from_rgb(150, 0, 0)));
                if sell_button.clicked() {
                    // TODO: Implement quick sell
                }
            });
        });
        
        ui.add_space(10.0);
        
        // Portfolio summary
        ui.group(|ui| {
            ui.label(RichText::new("💼 Portfolio").size(14.0).strong());
            ui.label(format!("Total Value: ${:.2}", self.total_portfolio_value));
            ui.label(format!("Daily P&L: {:.2}%", self.daily_pnl));
            ui.label(format!("Active Positions: {}", self.stocks_map.lock().unwrap().len()));
        });
    }

    fn show_charts_area(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.stocks_map.lock().unwrap().is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("📊 Add a stock symbol to start trading").size(16.0).color(Color32::GRAY));
            });
        } else {
            for (_, stock) in self.stocks_map.lock().unwrap().iter_mut() {
                create_new_stock_window(&mut stock.lock().unwrap(), ctx);
            }
        }
    }

    fn show_help_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("📖 Keyboard Shortcuts")
            .open(&mut self.show_help)
            .show(ctx, |ui| {
                ui.label(RichText::new("Keyboard Shortcuts").size(16.0).strong());
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("F1").monospace());
                    ui.label("Toggle this help window");
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Ctrl+Q").monospace());
                    ui.label("Quit application");
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Ctrl+N").monospace());
                    ui.label("New stock picker");
                });
                
                ui.separator();
                ui.label(RichText::new("Mouse Controls").size(14.0).strong());
                ui.label("• Drag to pan charts");
                ui.label("• Scroll to zoom");
                ui.label("• Right-click for context menu");
            });
    }
}

// fn plot(ui: &mut egui::Ui) -> egui::Response {

//     let n = 128;
//     let line_points: PlotPoints = (0..=n)
//         .map(|index| {
//             use std::f64::consts::TAU;
//             let x = egui::remap(index as f64, 0.0..=n as f64, -TAU..=TAU);
//             [x, x.sin()]
//         })
//         .collect();
//     let line = Line::new(line_points);
//     let box_elements = (0..=n)
//         .map(|i| {
//             use std::f64::consts::TAU;
//             let x = egui::remap(i as f64, 0.0..=n as f64, -TAU..=TAU);
//             let y = x.sin();
//             let spread = BoxSpread::new(y, y + 1.0, y + 2.0, y + 3.0, y + 4.0);
//             BoxElem::new(x, spread)
//         })
//         .collect();
//     let box_plot = BoxPlot::new(box_elements);
//     egui_plot::Plot::new("a plot")
//         .height(200.0)
//         .show_axes(true)
//         .data_aspect(1.0)
//         .show(ui, |plot_ui| {
//             plot_ui.line(line);
//             plot_ui.box_plot(box_plot);
//         })
//         .response
// }
