
use egui::{Color32, Stroke, Vec2};
use egui_plot::{BoxElem, BoxPlot, BoxSpread, PlotUi};
use egui_plot::{Line, PlotPoints};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rusty_trading_lib::structs::{Point, TimeRange, TimeSeries, Transaction};

use crate::TemplateApp;


#[derive(serde::Deserialize, serde::Serialize)]
pub struct Stock {
    candle_toggle: bool,
    line_toggle: bool,
    // managing the stock data, similar to value above
    time_series: Arc<Mutex<TimeSeries>>,
    // last time the data is updated
    last_update: DateTime<Utc>,
    stock_name: String,
    qty: String,
    price: String,
    open: bool,
}

impl Stock {
    pub fn default(stock_name: &str) -> Self {
        let time_series = TimeSeries::new(TimeRange::Day, Utc::now(), Utc::now(), vec![]);
        let time_series_arc = Arc::new(Mutex::new(time_series));
        Self {
            candle_toggle: true,
            line_toggle: false,
            time_series: time_series_arc,
            last_update: Utc::now(),
            stock_name: stock_name.to_owned(),
            qty: String::new(),
            price: String::new(),
            open: true,
        }
    }
}


pub fn create_new_stock_window(stock: &mut Stock, ctx: &egui::Context) {
    let mut line_toggle = stock.line_toggle;
    let mut candle_toggle = stock.candle_toggle;
    let qty = &mut stock.qty;
    let price = &mut stock.price;
    let stock_name = &stock.stock_name;

    egui::Window::new("Stock: ".to_owned() + stock_name)
        .open(&mut stock.open)
        .show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().text_edit_width = 50.;
            ui.label("Quantity:");
            ui.text_edit_singleline(qty);
            ui.label("Price:");
            ui.text_edit_singleline(price);
        });
        ui.horizontal(|ui| {
            let url = "http://127.0.0.1:3000/transaction";
            if ui.button("BUY").clicked() {

                let transaction = Transaction::buy(stock_name.to_owned(), price.parse::<f32>().unwrap(), qty.parse::<u32>().unwrap());
                let val = serde_json::to_value(transaction).unwrap();
                log::info!("{val}");
                let req = ehttp::Request::json(url, &val).unwrap();
                ehttp::fetch(req, move |response| {
                    log::info!("{:?}", response.unwrap().text().unwrap())
                });
            };
            if ui.button("SELL").clicked() {
                let transaction = Transaction::sell(stock_name.clone(), price.parse::<f32>().unwrap(), qty.parse::<u32>().unwrap());
                let val = serde_json::to_value(transaction).unwrap();
                log::info!("{val}");
                let req = ehttp::Request::json(url, &val).unwrap();
                ehttp::fetch(req, move |response| {
                    log::info!("{:?}", response.unwrap().text().unwrap())
                });
            };
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut candle_toggle, "Candle");
            ui.checkbox(&mut line_toggle, "Line");
        });
        // Add plot
        plot_stock(ui, &line_toggle, &candle_toggle, &mut stock.time_series);
        ui.add(egui::Hyperlink::from_label_and_url(
            "Source",
            "https://github.com/havvyliu/rusty-trading-egui",
        ));

    });

}

fn plot_stock(ui: &mut egui::Ui, line_toggle: &bool, candle_toggle: &bool, time_series: &mut Arc<Mutex<TimeSeries>>) -> egui::Response {

    egui_plot::Plot::new("stonk")
        .view_aspect(1.6)
        .min_size(Vec2::new(600.0, 200.0))
        .set_margin_fraction(Vec2::new(0.1, 0.1))
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_line(line_toggle, time_series, plot_ui);
            plot_candle(candle_toggle, time_series, plot_ui);
        })
        .response
}

fn plot_line(line_toggle: &bool, time_series: &mut Arc<Mutex<TimeSeries>>, plot_ui:&mut PlotUi) {
    if !line_toggle {return;}
    let points: Vec<Point> = time_series.lock().unwrap().data().into_iter()
        .map(|p: &Point| {
            Point::new(p.open, p.high, p.low, p.close, p.volume)
        })
        .collect();
    let len = points.len();
    let line_points: PlotPoints = (0..len)
        .map(|i| {
            [i as f64, points.get(i).unwrap().close as f64]
        })
        .collect();
    let line = Line::new(line_points);
    plot_ui.line(line);
}

fn plot_candle(candle_toggle: &bool, time_series: &mut Arc<Mutex<TimeSeries>>, plot_ui:&mut PlotUi) {
    if !candle_toggle {return;}
    let points: Vec<Point> = time_series.lock().unwrap().data().into_iter()
        .map(|p: &Point| {
            Point::new(p.open, p.high, p.low, p.close, p.volume)
        })
        .collect();
    let len = points.len();
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
            BoxElem::new(i as f64, spread)
                .box_width(1.)
                .stroke(Stroke::new(1., color))
                .whisker_width(0.5)
                .fill(color)
        })
        .collect();
    let box_plot = BoxPlot::new(box_elements);
    plot_ui.box_plot(box_plot);
}
