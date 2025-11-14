
use egui::{Color32, Stroke, Vec2, RichText, Frame, Margin, Rounding};
use egui_plot::{BoxElem, BoxPlot, BoxSpread, PlotUi, Plot};
use egui_plot::{Line, PlotPoints, Bar, BarChart};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rusty_trading_lib::structs::{Point, TimeRange, TimeSeries, Transaction};


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

fn call_simulate_v2(stock: &Stock) {
    let stock_name = stock.stock_name.clone();
    let url = format!("http://127.0.0.1:3000/simulate_v2?stock={}", stock_name);
    
    let req = ehttp::Request::json(url, "").unwrap();
    ehttp::fetch(req, move |response| {
        match response {
            Ok(resp) => log::info!("Simulation v2 for {} done...", stock_name),
            Err(e) => log::error!("Simulation v2 failed due to: {:?}", e),
        }
    });

}

pub fn create_new_stock_window(stock: &mut Stock, ctx: &egui::Context) {
    // Update mock data for demonstration
    update_mock_market_data(stock);

    call_simulate_v2(stock);
    
    let stock_name = stock.stock_name.clone();
    let mut open = stock.open;

    if let Some(response) = egui::Window::new(format!("📈 {}", stock_name))
        .open(&mut open)
        .frame(Frame::window(&ctx.style())
            .fill(Color32::from_rgb(20, 25, 30))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0)))
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
                        ui.checkbox(&mut stock.candle_toggle, "🕯️ Candles");
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
                        let buy_button = ui.add(egui::Button::new(RichText::new("🟢 BUY").color(Color32::WHITE))
                            .fill(Color32::from_rgb(0, 150, 0)));
                        if buy_button.clicked() {
                            if validate_trade_inputs(&stock.qty, &stock.price) {
                                stock.pending_order_type = "BUY".to_string();
                                stock.show_order_confirmation = true;
                            }
                        }
                        
                        let sell_button = ui.add(egui::Button::new(RichText::new("🔴 SELL").color(Color32::WHITE))
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
        .frame(Frame::window(&ctx.style()).fill(Color32::from_rgb(30, 35, 40)))
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
    let price = stock.price.parse::<f32>().unwrap();
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

fn plot_stock_enhanced(ui: &mut egui::Ui, stock: &mut Stock) -> egui::Response {
    let plot = Plot::new("enhanced_stock_plot")
        .view_aspect(2.0)
        .min_size(Vec2::new(200.0, 100.0))
        .set_margin_fraction(Vec2::new(0.05, 0.1))
        .show_axes(true)
        .allow_zoom(true)
        .allow_drag(true)
        .allow_scroll(true)
        .show_background(false)
        .show_x(true)
        .show_y(true);

    plot.show(ui, |plot_ui| {
        // Plot line chart if enabled
        if stock.line_toggle {
            plot_line(&stock.line_toggle, &mut stock.time_series, plot_ui);
        }
        
        // Plot candlestick chart if enabled
        if stock.candle_toggle {
            plot_candle(&stock.candle_toggle, &mut stock.time_series, plot_ui);
        }
        
        // Plot volume bars if enabled
        if stock.volume_toggle {
            plot_volume(&mut stock.time_series, plot_ui);
        }
        
        // Add crosshair and price indicators
        add_crosshair_and_indicators(plot_ui, stock.current_price);
    }).response
}

fn plot_volume(time_series: &mut Arc<Mutex<TimeSeries>>, plot_ui: &mut PlotUi) {
    let points: Vec<Point> = time_series.lock().unwrap().data().into_iter()
        .map(|p: &Point| Point::new(p.open, p.high, p.low, p.close, p.volume))
        .collect();
    
    let len = points.len();
    if len == 0 { return; }
    
    let volume_bars: Vec<Bar> = (0..len)
        .map(|i| {
            let point = points.get(i).unwrap();
            Bar::new(i as f64, point.volume as f64)
                .width(0.8)
                .fill(Color32::from_rgba_unmultiplied(100, 100, 100, 100))
        })
        .collect();
    
    let volume_chart = BarChart::new(volume_bars)
        .color(Color32::from_rgb(100, 100, 100))
        .name("Volume");
    
    plot_ui.bar_chart(volume_chart);
}

fn add_crosshair_and_indicators(plot_ui: &mut PlotUi, current_price: f32) {
    // Add horizontal line for current price
    let current_price_line = Line::new(PlotPoints::from(vec![
        [0.0, current_price as f64],
        [100.0, current_price as f64],
    ]))
    .color(Color32::from_rgb(255, 165, 0))
    .style(egui_plot::LineStyle::Dashed { length: 10.0 })
    .width(2.0)
    .name("Current Price");
    
    plot_ui.line(current_price_line);
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
