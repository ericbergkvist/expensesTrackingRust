#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

mod table;

use table::{Demo, TransactionTable};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 1024.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Expenses Tracking GUI",
        options,
        Box::new(|_cc| Ok(Box::<TransactionTableWindow>::default())),
    )
}

struct TransactionTableWindow {
    is_open: bool,
    table: TransactionTable,
}

impl Default for TransactionTableWindow {
    fn default() -> Self {
        Self {
            is_open: true,
            table: Default::default(),
        }
    }
}

impl eframe::App for TransactionTableWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //self.ui(ctx);
        self.table.show(ctx, &mut self.is_open);
    }
}
