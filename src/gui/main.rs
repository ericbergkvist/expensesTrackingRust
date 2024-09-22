#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use expenses_tracking::expense_tracker::ExpenseTracker;
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

struct ExpensesTrackingGUI {
    expenses_tracker: ExpenseTracker,
    name: String,
    age: u32,
}

impl Default for ExpensesTrackingGUI {
    fn default() -> Self {
        Self {
            expenses_tracker: ExpenseTracker::new(),
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
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

impl eframe::App for ExpensesTrackingGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let name_before_update = self.name.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.label("Text is rendered here");
            ui.heading("Expenses Tracking GUI");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });

        if self.name != name_before_update {
            println!("Name change to {}", self.name);
        }
    }
}
