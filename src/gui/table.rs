// Based on https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/table_demo.rs

use eframe::egui::{TextStyle, TextWrapMode};

/// Something to view in the demo windows
pub trait View {
    fn ui(&mut self, ui: &mut eframe::egui::Ui);
}

/// Something to view
pub trait Demo {
    /// Is the demo enabled for this integration?
    fn is_enabled(&self, _ctx: &eframe::egui::Context) -> bool {
        true
    }

    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &eframe::egui::Context, open: &mut bool);
}

#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum DemoType {
    Manual,
    ManyHomogeneous,
    ManyHeterogenous,
}

/// Shows off a table with dynamic layout
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TableDemo {
    demo: DemoType,
    striped: bool,
    resizable: bool,
    clickable: bool,
    num_rows: usize,
    scroll_to_row_slider: usize,
    scroll_to_row: Option<usize>,
    selection: std::collections::HashSet<usize>,
    checked: bool,
}

impl Default for TableDemo {
    fn default() -> Self {
        Self {
            demo: DemoType::Manual,
            striped: true,
            resizable: true,
            clickable: true,
            num_rows: 10_000,
            scroll_to_row_slider: 0,
            scroll_to_row: None,
            selection: Default::default(),
            checked: false,
        }
    }
}

impl Demo for TableDemo {
    fn name(&self) -> &'static str {
        "☰ Table"
    }

    fn show(&mut self, ctx: &eframe::egui::Context, open: &mut bool) {
        eframe::egui::Window::new(self.name())
            .open(open)
            .default_width(400.0)
            .show(ctx, |ui| {
                use View as _;
                self.ui(ui);
            });
    }
}

const NUM_MANUAL_ROWS: usize = 20;

impl View for TableDemo {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        let mut reset = false;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.striped, "Striped");
                ui.checkbox(&mut self.resizable, "Resizable columns");
                ui.checkbox(&mut self.clickable, "Clickable rows");
            });

            ui.label("Table type:");
            ui.radio_value(&mut self.demo, DemoType::Manual, "Few, manual rows");
            ui.radio_value(
                &mut self.demo,
                DemoType::ManyHomogeneous,
                "Thousands of rows of same height",
            );
            ui.radio_value(
                &mut self.demo,
                DemoType::ManyHeterogenous,
                "Thousands of rows of differing heights",
            );

            if self.demo != DemoType::Manual {
                ui.add(
                    eframe::egui::Slider::new(&mut self.num_rows, 0..=100_000)
                        .logarithmic(true)
                        .text("Num rows"),
                );
            }

            {
                let max_rows = if self.demo == DemoType::Manual {
                    NUM_MANUAL_ROWS
                } else {
                    self.num_rows
                };

                let slider_response = ui.add(
                    eframe::egui::Slider::new(&mut self.scroll_to_row_slider, 0..=max_rows)
                        .logarithmic(true)
                        .text("Row to scroll to"),
                );
                if slider_response.changed() {
                    self.scroll_to_row = Some(self.scroll_to_row_slider);
                }
            }

            reset = ui.button("Reset").clicked();
        });

        ui.separator();

        // Leave room for the source code link after the table demo:
        let body_text_size = TextStyle::Body.resolve(ui.style()).size;
        use egui_extras::{Size, StripBuilder};
        StripBuilder::new(ui)
            .size(Size::remainder().at_least(100.0)) // for the table
            .size(Size::exact(body_text_size)) // for the source code link
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    eframe::egui::ScrollArea::horizontal().show(ui, |ui| {
                        self.table_ui(ui, reset);
                    });
                });
                strip.cell(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(crate::egui_github_link_file!());
                    });
                });
            });
    }
}

impl TableDemo {
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
            .column(
                Column::remainder()
                    .at_least(40.0)
                    .clip(true)
                    .resizable(true),
            )
            .column(Column::auto())
            .column(Column::remainder())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        if self.clickable {
            table = table.sense(eframe::egui::Sense::click());
        }

        if let Some(row_index) = self.scroll_to_row.take() {
            table = table.scroll_to_row(row_index, None);
        }

        if reset {
            table.reset();
        }

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Row");
                });
                header.col(|ui| {
                    ui.strong("Clipped text");
                });
                header.col(|ui| {
                    ui.strong("Expanding content");
                });
                header.col(|ui| {
                    ui.strong("Interaction");
                });
                header.col(|ui| {
                    ui.strong("Content");
                });
            })
            .body(|mut body| match self.demo {
                DemoType::Manual => {
                    for row_index in 0..NUM_MANUAL_ROWS {
                        let is_thick = thick_row(row_index);
                        let row_height = if is_thick { 30.0 } else { 18.0 };
                        body.row(row_height, |mut row| {
                            row.set_selected(self.selection.contains(&row_index));

                            row.col(|ui| {
                                ui.label(row_index.to_string());
                            });
                            row.col(|ui| {
                                ui.label(long_text(row_index));
                            });
                            row.col(|ui| {
                                expanding_content(ui);
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut self.checked, "Click me");
                            });
                            row.col(|ui| {
                                ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                                if is_thick {
                                    ui.heading("Extra thick row");
                                } else {
                                    ui.label("Normal row");
                                }
                            });

                            self.toggle_row_selection(row_index, &row.response());
                        });
                    }
                }
                DemoType::ManyHomogeneous => {
                    body.rows(text_height, self.num_rows, |mut row| {
                        let row_index = row.index();
                        row.set_selected(self.selection.contains(&row_index));

                        row.col(|ui| {
                            ui.label(row_index.to_string());
                        });
                        row.col(|ui| {
                            ui.label(long_text(row_index));
                        });
                        row.col(|ui| {
                            expanding_content(ui);
                        });
                        row.col(|ui| {
                            ui.checkbox(&mut self.checked, "Click me");
                        });
                        row.col(|ui| {
                            ui.add(
                                eframe::egui::Label::new("Thousands of rows of even height")
                                    .wrap_mode(TextWrapMode::Extend),
                            );
                        });

                        self.toggle_row_selection(row_index, &row.response());
                    });
                }
                DemoType::ManyHeterogenous => {
                    let row_height = |i: usize| if thick_row(i) { 30.0 } else { 18.0 };
                    body.heterogeneous_rows((0..self.num_rows).map(row_height), |mut row| {
                        let row_index = row.index();
                        row.set_selected(self.selection.contains(&row_index));

                        row.col(|ui| {
                            ui.label(row_index.to_string());
                        });
                        row.col(|ui| {
                            ui.label(long_text(row_index));
                        });
                        row.col(|ui| {
                            expanding_content(ui);
                        });
                        row.col(|ui| {
                            ui.checkbox(&mut self.checked, "Click me");
                        });
                        row.col(|ui| {
                            ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                            if thick_row(row_index) {
                                ui.heading("Extra thick row");
                            } else {
                                ui.label("Normal row");
                            }
                        });

                        self.toggle_row_selection(row_index, &row.response());
                    });
                }
            });
    }

    fn toggle_row_selection(&mut self, row_index: usize, row_response: &eframe::egui::Response) {
        if row_response.clicked() {
            if self.selection.contains(&row_index) {
                self.selection.remove(&row_index);
            } else {
                self.selection.insert(row_index);
            }
        }
    }
}

fn expanding_content(ui: &mut eframe::egui::Ui) {
    ui.add(eframe::egui::Separator::default().horizontal());
}

fn long_text(row_index: usize) -> String {
    format!("Row {row_index} has some long text that you may want to clip, or it will take up too much horizontal space!")
}

fn thick_row(row_index: usize) -> bool {
    row_index % 6 == 0
}

/// Create a [`Hyperlink`](egui::Hyperlink) to this egui source code file on github.
#[macro_export]
macro_rules! egui_github_link_file {
    () => {
        $crate::egui_github_link_file!("(source code)")
    };
    ($label: expr) => {
        eframe::egui::github_link_file!(
            "https://github.com/emilk/egui/blob/master/",
            eframe::egui::RichText::new($label).small()
        )
    };
}
