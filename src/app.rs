use std::{collections::HashMap, iter::Map, sync::{Arc, Mutex}, time::Duration};
use egui::{Color32, Stroke, Vec2};
use egui_plot::{BoxElem, BoxPlot, PlotUi};
use egui_plot::{Line, PlotPoints};

use chrono::{DateTime, Utc};
use egui_plot::BoxSpread;
use rusty_trading_lib::structs::{Point, TimeRange, TimeSeries, Transaction};

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
}

impl Default for TemplateApp {
    fn default() -> Self {
        let app = Self {
            label: "Hello World!".to_owned(),
            candle_toggle: true,
            line_toggle: false,
            value: Arc::new(Mutex::new(2.7)),
            last_update: Utc::now(),
            stock: String::new(),
            qty: String::new(),
            price: String::new(),
            stocks_map: Arc::new(Mutex::new(HashMap::new())),
        };
        app
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        let now = Utc::now();
        if self.last_update + Duration::from_secs(100) <= now {
            let ctx_clone = ctx.clone();
            let mut map = self.stocks_map.lock().unwrap();
            for (key, val) in map.iter_mut() {
                let request_template = ehttp::Request::get(format!("http://127.0.0.1:3000/daily?stock={key}"));
                log::info!("now is {:?}", Utc::now());
                log::info!("calling get_daily api and repaint graph");
                let val_clone = Arc::clone(&val);
                ehttp::fetch(request_template, move |result: ehttp::Result<ehttp::Response>| {
                    let time_series: TimeSeries = serde_json::from_slice(&result.unwrap().bytes).unwrap();
                    val_clone.lock().unwrap().set_time_series(time_series);
                });
            }
            ctx_clone.request_repaint();
            self.last_update = now;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::Window::new("Stock Picker").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().text_edit_width = 50.;
                ui.label("Stock:");
                ui.text_edit_singleline(&mut self.stock);
            });
            ui.horizontal(|ui| {
                if ui.button("PICK").clicked() {
                    self.stocks_map.lock().unwrap().insert(self.stock.clone(), 
                        Arc::new(Mutex::new(Stock::default(&self.stock))));
                }
            });
        });

        for (_, stock) in self.stocks_map.lock().unwrap().iter_mut() {
            create_new_stock_window(&mut stock.lock().unwrap(), ctx);
        }
    }
}

fn plot(ui: &mut egui::Ui) -> egui::Response {

    let n = 128;
    let line_points: PlotPoints = (0..=n)
        .map(|index| {
            use std::f64::consts::TAU;
            let x = egui::remap(index as f64, 0.0..=n as f64, -TAU..=TAU);
            [x, x.sin()]
        })
        .collect();
    let line = Line::new(line_points);
    let box_elements = (0..=n)
        .map(|i| {
            use std::f64::consts::TAU;
            let x = egui::remap(i as f64, 0.0..=n as f64, -TAU..=TAU);
            let y = x.sin();
            let spread = BoxSpread::new(y, y + 1.0, y + 2.0, y + 3.0, y + 4.0);
            BoxElem::new(x, spread)
        })
        .collect();
    let box_plot = BoxPlot::new(box_elements);
    egui_plot::Plot::new("a plot")
        .height(200.0)
        .show_axes(true)
        .data_aspect(1.0)
        .show(ui, |plot_ui| {
            plot_ui.line(line);
            plot_ui.box_plot(box_plot);
        })
        .response
}
