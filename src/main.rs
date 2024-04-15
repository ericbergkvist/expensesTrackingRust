use std::{error::Error, path::PathBuf, str::FromStr};

mod expense_tracker;
mod transaction;

use expense_tracker::ExpenseTracker;

fn main() -> Result<(), Box<dyn Error>> {
    // Enable logging
    env_logger::init();

    // Specify the path to your CSV file
    let transactions_file_path = PathBuf::from_str("/Users/eric/Desktop/transactions_short.csv")
        .map_err(|e| format!("Failed to convert path of input transactions CSV file: {e}"))?;

    let mut expense_tracker = ExpenseTracker::new();
    expense_tracker.load_transactions_from_file(&transactions_file_path, true)?;

    //println!("{:?}", expense_tracker);

    let output_path = PathBuf::from_str("output.csv")
        .map_err(|e| format!("Failed to create output path: {e}"))?;
    expense_tracker
        .write_transactions_to_file(&output_path)
        .map_err(|e| format!("Failed to write transaction to CSV file: {e}"))?;

    let config_file_path = PathBuf::from_str("config/expenseTrackerConfig.json")
        .map_err(|e| format!("Failed to open config file at: {e}"))?;

    expense_tracker.save_info_to_file(config_file_path)?;

    Ok(())
}
