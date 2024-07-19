#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod stock;
pub use app::TemplateApp;
pub use stock::Stock;
pub use stock::create_new_stock_window;
