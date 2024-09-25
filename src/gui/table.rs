// Based on https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/table_demo.rs

use expenses_tracking::{
    expense_tracker::ExpenseTracker,
    transaction::{self, Category, Transaction},
};

use std::{path::PathBuf, str::FromStr};

/// Something to view.
pub trait Widget {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Shows the state of the widget.
    fn show(&mut self, ctx: &eframe::egui::Context, open: &mut bool);

    /// What to display in the widget.
    fn ui(&mut self, ui: &mut eframe::egui::Ui);
}

/// Shows off a table with dynamic layout
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TransactionTable {
    striped: bool,
    resizable: bool,
    expense_tracker: ExpenseTracker,
    transaction_category_filter: CategoryFilter,
}

#[derive(PartialEq)]
enum CategoryFilter {
    NoneSelected,
    CategorySelected(Category),
}

impl Default for TransactionTable {
    fn default() -> Self {
        Self {
            striped: true,
            resizable: true,
            expense_tracker: ExpenseTracker::new(),
            transaction_category_filter: CategoryFilter::NoneSelected,
        }
    }
}

impl Widget for TransactionTable {
    fn name(&self) -> &'static str {
        "☰ Transactions"
    }

    fn show(&mut self, ctx: &eframe::egui::Context, open: &mut bool) {
        eframe::egui::Window::new(self.name())
            .open(open)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        let mut reset = false;
        let mut load_transactions = false;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.striped, "Striped");
                ui.checkbox(&mut self.resizable, "Resizable columns");
            });
            reset = ui.button("Reset").clicked();
            load_transactions = ui.button("Load transactions").clicked();
        });

        ui.horizontal(|ui| {
            eframe::egui::ComboBox::from_label("Category filter")
                .selected_text(match &self.transaction_category_filter {
                    CategoryFilter::NoneSelected => "None".to_string(),
                    CategoryFilter::CategorySelected(category) => category.name.clone(),
                })
                .show_ui(ui, |ui| {
                    // Add "None" as an option
                    ui.selectable_value(
                        &mut self.transaction_category_filter,
                        CategoryFilter::NoneSelected,
                        "None",
                    );

                    // Add valid categories as options
                    for category in &self.expense_tracker.valid_categories {
                        ui.selectable_value(
                            &mut self.transaction_category_filter,
                            CategoryFilter::CategorySelected(category.clone()),
                            &category.name,
                        );
                    }
                });
        });

        if load_transactions {
            let transactions_file_path =
                PathBuf::from_str("/Users/eric/Desktop/transactions_short.csv")
                    .map_err(|e| {
                        format!("Failed to convert path of input transactions CSV file: {e}")
                    })
                    .unwrap();

            let mut expense_tracker = ExpenseTracker::new();
            expense_tracker
                .load_transactions_from_file(&transactions_file_path, true)
                .unwrap();

            self.expense_tracker = expense_tracker;
        }

        ui.separator();

        use egui_extras::{Size, StripBuilder};
        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0)) // for the table
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    eframe::egui::ScrollArea::horizontal().show(ui, |ui| {
                        self.table_ui(ui, reset);
                    });
                });
            });
    }
}

impl TransactionTable {
    fn table_ui(&mut self, ui: &mut eframe::egui::Ui, reset: bool) {
        use egui_extras::{Column, TableBuilder};

        let text_height = eframe::egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let available_height = ui.available_height();
        let mut table = TableBuilder::new(ui)
            .striped(self.striped)
            .resizable(self.resizable)
            .cell_layout(eframe::egui::Layout::left_to_right(
                eframe::egui::Align::Center,
            ))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(
                Column::remainder()
                    .at_least(40.0)
                    .clip(true)
                    .resizable(true),
            )
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        if reset {
            table.reset();
        }

        let transactions = if self.transaction_category_filter == CategoryFilter::NoneSelected {
            self.expense_tracker.transactions.clone()
        } else {
            self.expense_tracker
                .transactions
                .iter()
                .filter(|transaction| match &self.transaction_category_filter {
                    CategoryFilter::NoneSelected => true,
                    CategoryFilter::CategorySelected(category) => {
                        transaction.category_name.to_lowercase() == category.name
                    }
                })
                .cloned()
                .collect()
        };

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("ID");
                });
                header.col(|ui| {
                    ui.strong("Date");
                });
                header.col(|ui| {
                    ui.strong("Amount paid");
                });
                header.col(|ui| {
                    ui.strong("Amount received");
                });
                header.col(|ui| {
                    ui.strong("Category");
                });
                header.col(|ui| {
                    ui.strong("Sub-category");
                });
                header.col(|ui| {
                    ui.strong("Tag");
                });
                header.col(|ui| {
                    ui.strong("Note");
                });
            })
            .body(|body| {
                body.rows(text_height, transactions.len(), |mut row| {
                    let row_index = row.index();
                    let transaction = &transactions[row_index];
                    let amount = transaction.amount;
                    let (mut amount_in, mut amount_out) = (0.0, 0.0);
                    if amount > 0.0 {
                        amount_in = amount;
                    } else {
                        amount_out = -amount;
                    }

                    row.col(|ui| {
                        ui.label(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.label(transaction.date.to_string());
                    });
                    row.col(|ui| {
                        ui.label(amount_out.to_string());
                    });
                    row.col(|ui| {
                        ui.label(amount_in.to_string());
                    });
                    row.col(|ui| {
                        ui.label(transaction.category_name.as_str());
                    });
                    row.col(|ui| {
                        if let Some(subcategory_name) = &transaction.subcategory_name {
                            ui.label(subcategory_name.as_str());
                        } else {
                            ui.label("");
                        }
                    });
                    row.col(|ui| {
                        if let Some(tag) = &transaction.tag {
                            ui.label(tag.as_str());
                        } else {
                            ui.label("");
                        }
                    });
                    row.col(|ui| {
                        if let Some(note) = &transaction.note {
                            ui.label(note.as_str());
                        } else {
                            ui.label("");
                        }
                    });
                })
            })
    }
}
