
use egui::{Color32, Frame, Margin, RichText, Rounding, Stroke, Theme, Vec2};
use egui_plot::{Bar, BarChart, BoxElem, BoxPlot, BoxSpread, GridMark, Line, Plot, PlotPoints, PlotUi};
use std::{ops::RangeInclusive, sync::{Arc, Mutex}};
use chrono::{DateTime, TimeZone, Utc};
use rusty_trading_model::structs::{Point, TimeRange, TimeSeries, Transaction};


#[derive(serde::Deserialize, serde::Serialize)]
pub struct Stock {
    candle_toggle: bool,
    line_toggle: bool,
    volume_toggle: bool,
    // managing the stock data, similar to value above
    time_series: Arc<Mutex<TimeSeries>>,
    // last time the data is updated
    last_update: DateTime<Utc>,
    stock_name: String,
    qty: String,
    price: String,
    open: bool,
    // New fields for enhanced trading
    #[serde(skip)]
    current_price: f32,
    #[serde(skip)]
    bid_price: f32,
    #[serde(skip)]
    ask_price: f32,
    #[serde(skip)]
    daily_change: f32,
    #[serde(skip)]
    daily_change_percent: f32,
    #[serde(skip)]
    volume: u64,
    #[serde(skip)]
    show_order_confirmation: bool,
    #[serde(skip)]
    pending_order_type: String,
}

impl Stock {
    pub fn default(stock_name: &str) -> Self {
        let time_series = TimeSeries::new(TimeRange::Day, Utc::now(), Utc::now(), vec![]);
        let time_series_arc = Arc::new(Mutex::new(time_series));
        Self {
            candle_toggle: true,
            line_toggle: false,
            volume_toggle: true,
            time_series: time_series_arc,
            last_update: Utc::now(),
            stock_name: stock_name.to_owned(),
            qty: String::new(),
            price: String::new(),
            open: true,
            current_price: 0.0,
            bid_price: 0.0,
            ask_price: 0.0,
            daily_change: 0.0,
            daily_change_percent: 0.0,
            volume: 0,
            show_order_confirmation: false,
            pending_order_type: String::new(),
        }
    }

    pub fn set_time_series(self: &Self, time_series: TimeSeries) {
        *self.time_series.lock().unwrap() = time_series;
    }
}

fn call_start_simulation(stock: &Stock) {
    let stock_name = stock.stock_name.clone();
    let url = format!("http://127.0.0.1:3000/simulation_start?stock={}", stock_name);
    
    let req = ehttp::Request::json(url, "").unwrap();
    ehttp::fetch(req, move |response| {
        match response {
            Ok(resp) => log::info!("Simulation for {} done...", stock_name),
            Err(e) => log::error!("Simulation failed due to: {:?}", e),
        }
    });

}

pub fn create_new_stock_window(stock: &mut Stock, ctx: &egui::Context) {
    // Update mock data for demonstration
    update_mock_market_data(stock);

    call_start_simulation(&stock);
    
    let stock_name = stock.stock_name.clone();
    let mut open = stock.open;

    if let Some(response) = egui::Window::new(format!("📈 {}", stock_name))
        .open(&mut open)
        .min_size(Vec2::new(150.0, 100.0))
        .show(ctx, |ui| {
            // Header with stock info
            ui.horizontal(|ui| {
                ui.label(RichText::new(&stock_name).size(20.0).strong().color(Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let change_color = if stock.daily_change >= 0.0 {
                        Color32::from_rgb(0, 255, 0)
                    } else {
                        Color32::from_rgb(255, 0, 0)
                    };
                    ui.label(RichText::new(format!("{:.2}%", stock.daily_change_percent)).color(change_color));
                    ui.label(RichText::new(format!("${:.2}", stock.daily_change)).color(change_color));
                    ui.label(RichText::new(format!("${:.2}", stock.current_price)).size(16.0).strong());
                });
            });
            
            ui.separator();
            
            // Market data row
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.label(RichText::new("📊 Market Data").strong());
                    ui.horizontal(|ui| {
                        ui.label(format!("Bid: ${:.2}", stock.bid_price));
                        ui.separator();
                        ui.label(format!("Ask: ${:.2}", stock.ask_price));
                        ui.separator();
                        ui.label(format!("Vol: {}", format_volume(stock.volume)));
                    });
                });

                ui.group(|ui| {
                    ui.label(RichText::new("📊 Chart Options").strong());
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut stock.candle_toggle, "🕯 Candles");
                        ui.checkbox(&mut stock.line_toggle, "📈 Line");
                        ui.checkbox(&mut stock.volume_toggle, "📊 Volume");
                    });
                });
            });
            
            ui.add_space(8.0);
            
            // Trading controls
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.label(RichText::new("💰 Trade").strong());
                    ui.horizontal(|ui| {
                        ui.label("Qty:");
                        ui.add(egui::TextEdit::singleline(&mut stock.qty).desired_width(60.0));
                        ui.label("Price:");
                        ui.add(egui::TextEdit::singleline(&mut stock.price).desired_width(80.0));
                    });
                    
                    ui.horizontal(|ui| {
                        let buy_button = ui.add(egui::Button::new(RichText::new("BUY").color(Color32::WHITE))
                            .fill(Color32::from_rgb(0, 150, 0)));
                        if buy_button.clicked() {
                            if validate_trade_inputs(&stock.qty, &stock.price) {
                                stock.pending_order_type = "BUY".to_string();
                                stock.show_order_confirmation = true;
                            }
                        }
                        
                        let sell_button = ui.add(egui::Button::new(RichText::new("SELL").color(Color32::WHITE))
                            .fill(Color32::from_rgb(150, 0, 0)));
                        if sell_button.clicked() {
                            if validate_trade_inputs(&stock.qty, &stock.price) {
                                stock.pending_order_type = "SELL".to_string();
                                stock.show_order_confirmation = true;
                            }
                        }
                        
                        if ui.button("📋 Market").on_hover_text("Buy/Sell at market price").clicked() {
                            stock.price = stock.current_price.to_string();
                        }
                    });
                });
                
                
            });
            
            ui.separator();
            
            // Enhanced plot
            plot_stock_enhanced(ui, stock);
        }) {
        // Update the open state
        stock.open = open;
    }
    
    // Order confirmation dialog (outside the main window to avoid borrowing issues)
    if stock.show_order_confirmation {
        show_order_confirmation_dialog(stock, ctx);
    }
}

fn update_mock_market_data(stock: &mut Stock) {
    // Simulate real-time market data updates
    use std::f32::consts::PI;
    let time_factor = (Utc::now().timestamp() as f32 / 10.0) % (2.0 * PI);
    
    stock.current_price = 175.5 + (time_factor.sin() * 5.0);
    stock.bid_price = stock.current_price - 0.05;
    stock.ask_price = stock.current_price + 0.05;
    stock.daily_change = time_factor.sin() * 2.5;
    stock.daily_change_percent = (stock.daily_change / stock.current_price) * 100.0;
    stock.volume = 1_250_000 + ((time_factor * 1000.0) as u64);
}

fn format_volume(volume: u64) -> String {
    if volume >= 1_000_000 {
        format!("{:.1}M", volume as f64 / 1_000_000.0)
    } else if volume >= 1_000 {
        format!("{:.1}K", volume as f64 / 1_000.0)
    } else {
        volume.to_string()
    }
}

fn validate_trade_inputs(qty: &str, price: &str) -> bool {
    qty.parse::<u32>().is_ok() && price.parse::<f32>().is_ok()
}

fn show_order_confirmation_dialog(stock: &mut Stock, ctx: &egui::Context) {
    egui::Window::new("🔔 Confirm Order")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(RichText::new("Order Confirmation").size(16.0).strong());
            ui.separator();
            
            ui.label(format!("Symbol: {}", stock.stock_name));
            ui.label(format!("Type: {}", stock.pending_order_type));
            ui.label(format!("Quantity: {}", stock.qty));
            ui.label(format!("Price: ${}", stock.price));
            
            let total = stock.qty.parse::<u32>().unwrap_or(0) as f32 * stock.price.parse::<f32>().unwrap_or(0.0);
            ui.label(format!("Total: ${:.2}", total));
            
            ui.separator();
            
            ui.horizontal(|ui| {
                let confirm_button = ui.add(egui::Button::new(RichText::new("✅ Confirm").color(Color32::WHITE))
                    .fill(Color32::from_rgb(0, 150, 0)));
                if confirm_button.clicked() {
                    execute_trade(stock);
                    stock.show_order_confirmation = false;
                }
                
                let cancel_button = ui.add(egui::Button::new(RichText::new("❌ Cancel").color(Color32::WHITE))
                    .fill(Color32::from_rgb(150, 0, 0)));
                if cancel_button.clicked() {
                    stock.show_order_confirmation = false;
                }
            });
        });
}

fn execute_trade(stock: &mut Stock) {
    let url = "http://127.0.0.1:3000/transaction";
    let stock_name = stock.stock_name.clone();
    let price = stock.price.parse::<f64>().unwrap();
    let qty = stock.qty.parse::<u32>().unwrap();
    
    let transaction = if stock.pending_order_type == "BUY" {
        Transaction::buy(stock_name, price, qty)
    } else {
        Transaction::sell(stock_name, price, qty)
    };
    
    let val = serde_json::to_value(transaction).unwrap();
    log::info!("Executing trade: {val}");
    let req = ehttp::Request::json(url, &val).unwrap();
    ehttp::fetch(req, move |response| {
        match response {
            Ok(resp) => log::info!("Trade executed successfully: {:?}", resp.text().unwrap()),
            Err(e) => log::error!("Trade execution failed: {:?}", e),
        }
    });
    
    // Clear form after successful submission
    stock.qty.clear();
    stock.price.clear();
}

fn plot_stock_enhanced(ui: &mut egui::Ui, stock: &mut Stock) -> egui::Response {
    let points = collect_time_series_points(&stock.time_series);
    let time_step = estimate_time_step(&points);

    let plot = Plot::new("enhanced_stock_plot")
        .view_aspect(2.0)
        .min_size(Vec2::new(200.0, 100.0))
        .set_margin_fraction(Vec2::new(0.05, 0.1))
        .show_axes(true)
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .show_background(false)
        .show_grid(false)
        .x_axis_formatter(format_time_axis)
        .show_x(true)
        .show_y(true);

    plot.show(ui, |plot_ui| {
        // Plot line chart if enabled
        if stock.line_toggle {
            plot_line(&points, plot_ui);
        }
        
        // Plot candlestick chart if enabled
        if stock.candle_toggle {
            plot_candle(&points, plot_ui, time_step);
        }
        
        // Plot volume bars if enabled
        if stock.volume_toggle {
            plot_volume(&points, plot_ui, time_step);
        }
    }).response
}

fn collect_time_series_points(time_series: &Arc<Mutex<TimeSeries>>) -> Vec<Point> {
    let mut guard = time_series.lock().unwrap();
    guard.data().clone()
}

fn timestamp_to_f64(timestamp: &DateTime<Utc>) -> f64 {
    timestamp.timestamp_millis() as f64
}

fn estimate_time_step(points: &[Point]) -> f64 {
    const DEFAULT_STEP_MS: f64 = 60_000.0;

    if points.len() < 2 {
        return DEFAULT_STEP_MS;
    }

    let mut total = 0.0;
    let mut count = 0;

    for window in points.windows(2) {
        let diff = window[1]
            .timestamp
            .signed_duration_since(window[0].timestamp)
            .num_milliseconds()
            .abs() as f64;

        if diff > 0.0 {
            total += diff;
            count += 1;
        }
    }

    if count > 0 {
        (total / count as f64).max(1.0)
    } else {
        DEFAULT_STEP_MS
    }
}

fn time_bounds(points: &[Point]) -> Option<(f64, f64)> {
    points.iter().map(|p| timestamp_to_f64(&p.timestamp)).fold(
        None,
        |acc, ts| match acc {
            Some((min_ts, max_ts)) => Some((min_ts.min(ts), max_ts.max(ts))),
            None => Some((ts, ts)),
        },
    )
}

fn format_time_axis(mark: GridMark, range: &RangeInclusive<f64>) -> String {
    if !mark.value.is_finite() {
        return String::new();
    }

    let span_ms = (*range.end() - *range.start()).abs();
    let date_time = Utc.timestamp_millis(mark.value as i64);

    const DAY_MS: f64 = 86_400_000.0;
    const HOUR_MS: f64 = 3_600_000.0;

    if span_ms > DAY_MS {
        date_time.format("%Y-%m-%d").to_string()
    } else if span_ms > HOUR_MS {
        date_time.format("%H:%M").to_string()
    } else {
        date_time.format("%H:%M:%S").to_string()
    }
}

fn plot_volume(points: &[Point], plot_ui: &mut PlotUi, time_step: f64) {
    if points.is_empty() {
        return;
    }

    let bar_width = (time_step * 0.6).max(1.0);

    let volume_bars: Vec<Bar> = points
        .iter()
        .map(|point| {
            Bar::new(timestamp_to_f64(&point.timestamp), point.volume as f64)
                .width(bar_width)
                .fill(Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        })
        .collect();
    
    let volume_chart = BarChart::new("Bar", volume_bars)
        .color(Color32::from_rgb(100, 100, 100))
        .name("Volume");
    
    plot_ui.bar_chart(volume_chart);
}

fn plot_line(points: &[Point], plot_ui: &mut PlotUi) {
    if points.is_empty() {
        return;
    }

    let line_points: PlotPoints = points
        .iter()
        .map(|point| [timestamp_to_f64(&point.timestamp), point.close as f64])
        .collect();

    let line = Line::new("LINE", line_points);
    plot_ui.line(line);
}

fn plot_candle(points: &[Point], plot_ui: &mut PlotUi, time_step: f64) {
    if points.is_empty() {
        return;
    }

    let candle_width = (time_step * 0.6).max(1.0);
    let whisker_width = (candle_width * 0.4).max(1.0);
    let mut box_elements = Vec::with_capacity(points.len());
    let mut previous_close: Option<f64> = None;

    for point in points {
        let spread = BoxSpread::new(point.low, point.open, point.close, point.close, point.high);
        let mut color = Color32::LIGHT_GREEN;

        if let Some(last_close) = previous_close {
            if last_close > point.close {
                color = Color32::LIGHT_RED;
            }
        }

        previous_close = Some(point.close);

        box_elements.push(
            BoxElem::new(timestamp_to_f64(&point.timestamp), spread)
                .box_width(candle_width)
                .stroke(Stroke::new(2., color))
                .whisker_width(whisker_width)
                .fill(color),
        );
    }
    let formatter = Box::new(|elem: &BoxElem, _plot: &BoxPlot| {
        let spread = &elem.spread;
        format!(
            "Open: {open:.2}\nClose: {close:.2}\nLow: {low:.2}\nHigh: {high:.2}",
            open = spread.quartile3,
            close = spread.quartile1,
            low = spread.lower_whisker,
            high = spread.upper_whisker,
        )
    });

    let box_plot = 
        BoxPlot::new("CANDLE", box_elements)
            .element_formatter(formatter);
    plot_ui.box_plot(box_plot);
}


//TODO: Enhance the BoxElem so tht it has better tooltips
