use std::error::Error;

mod expense_tracker;
mod transaction;

use expense_tracker::ExpenseTracker;

fn main() -> Result<(), Box<dyn Error>> {
    // Enable logging
    env_logger::init();

    // Specify the path to your CSV file
    //let file_path = "/home/ericbergkvist/personal/expensesTrackingRust/transactions.csv";
    let transactions_file_path = "/Users/eric/Desktop/transactions_short.csv";

    let mut expense_tracker = ExpenseTracker::new();
    expense_tracker.load_transactions_from_file(transactions_file_path, true, true)?;

    //println!("{:?}", expense_tracker);

    let config_file_path = "config/expenseTrackerConfig.json";
    expense_tracker.save_info_to_file(config_file_path)?;

    Ok(())
}
