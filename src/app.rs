use std::{sync::{Arc, Mutex}, time::Duration};
use egui::{Color32, Stroke};
use egui_plot::{BoxElem, BoxPlot};
use egui_plot::{Line, PlotPoints};

use chrono::{DateTime, Utc};
use egui_plot::BoxSpread;
use rusty_trading_lib::structs::{TimeRange, TimeSeries, Point};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: Arc<Mutex<f32>>,

    // managing the stock data, similar to value above
    time_series: Arc<Mutex<TimeSeries>>,
    // last time the data is updated
    last_update: DateTime<Utc>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let time_series = TimeSeries::new(TimeRange::Day, Utc::now(), Utc::now(), vec![]);
        let time_series_arc = Arc::new(Mutex::new(time_series));
        let app = Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: Arc::new(Mutex::new(2.7)),
            time_series: time_series_arc,
            last_update: Utc::now(),
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

        let request = ehttp::Request::get("http://127.0.0.1:3000/daily");
        let simulate_request = ehttp::Request::post("http://127.0.0.1:3000/simulate", "AMD".as_bytes().to_owned());
        let time_series_clone = self.time_series.clone();
        let ctx_clone = ctx.clone();
        let now = Utc::now();
        if self.last_update + Duration::from_secs(10) <= now {
            log::info!("now is {:?}", Utc::now());
            let _ = ehttp::fetch(simulate_request, |_| {});
            log::info!("calling get_daily api and repaint graph");
            ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                let time_series: TimeSeries = serde_json::from_slice(&result.unwrap().bytes).unwrap();
                *time_series_clone.lock().unwrap() = time_series;
                ctx_clone.request_repaint();
            });
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

        egui::Window::new("Rusty Trading").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut *self.value.lock().unwrap(), 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *self.value.lock().unwrap()  += 1.0;
            }
            ui.separator();

            // Add plot
            plot_stock(ui, self);

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/havvyliu/rusty-trading-egui",
                "Source code"
            ));

        });
    }
}

fn plot_stock(ui: &mut egui::Ui, app: &mut TemplateApp) -> egui::Response {
    let points: Vec<Point> = app.time_series.lock().unwrap().data().into_iter()
        .map(|p: &Point| {
            Point::new(p.open, p.high, p.low, p.close, p.volume)
        })
        .collect();
    let len = points.len();
    // let line_points: PlotPoints = (0..len)
    //     .map(|i| {
    //         [i as f64, points.get(i).unwrap().volume as f64]
    //     })
    //     .collect();
    // let line = Line::new(line_points);
    let box_elements = (1..len)
        .map(|i| {
            let point = points.get(i).unwrap();
            let spread = BoxSpread::new(point.low as f64, point.open as f64, point.close as f64, point.close as f64, point.high as f64);
            let mut color = Color32::LIGHT_GREEN;
            if let Some(last_point) = points.get(i - 1) {
                if last_point.close > point.close {
                    color = Color32::LIGHT_RED;
                }
            }
            BoxElem::new(i as f64, spread).stroke(Stroke::new(0.5, color))
                .fill(color)
        })
        .collect();
    let box_plot = BoxPlot::new(box_elements);
    egui_plot::Plot::new("a plot")
        .height(200.0)
        .width(600.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            // plot_ui.line(line);
            plot_ui.box_plot(box_plot)
        })
        .response
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

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
