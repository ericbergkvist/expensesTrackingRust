use core::f32;
use std::error::Error;
use std::fs::File;

mod expense_tracker;
mod transaction;

use expense_tracker::ExpenseTracker;
use transaction::Transaction;

fn main() -> Result<(), Box<dyn Error>> {
    // Specify the path to your CSV file
    //let file_path = "/home/ericbergkvist/personal/expensesTrackingRust/transactions.csv";
    let file_path = "/Users/eric/Desktop/transactions.csv";

    // Open the CSV file
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error while loading the CSV of transactions.");
            return Err(e.into());
        }
    };

    // Create a CSV reader
    let mut rdr = csv::Reader::from_reader(file);

    // Read the header record (first line)
    if let Ok(header) = rdr.headers() {
        // Convert the header entries into a Vec<String>
        let header_list: Vec<String> = header.iter().map(|entry| entry.to_string()).collect();

        // Print the resulting Vec<String>
        println!("{:?}", header_list);
    } else {
        eprintln!("CSV file does not have a header row.");
    }

    let mut n_lines: i32 = 0;
    let mut n_ignored_transactions: i32 = 0;

    let mut expense_tracker = ExpenseTracker::new();
    expense_tracker.add_category("Nourriture");
    expense_tracker.add_subcategory("Nourriture", "Courses")?;
    expense_tracker.add_subcategory("Nourriture", "Restaurant")?;

    // Iterate over each record in the CSV file
    for record in rdr.records() {
        // Handle each CSV record
        let csv_row = record?;

        let transaction = Transaction::from_csv_row(csv_row)?;

        match expense_tracker.add_transaction(transaction) {
            Ok(()) => (),
            Err(e) => {
                println!("{}", e);
                n_ignored_transactions += 1;
            }
        };

        n_lines += 1;
    }

    let sum_transactions: f32 = expense_tracker
        .transactions
        .iter()
        .map(|transaction| transaction.amount)
        .sum();
    println!("Sum of all transaction: {} CHF", sum_transactions);

    println!(
        "Number of lines in the CSV (excluding the header): {}",
        n_lines
    );
    println!(
        "Number of valid transactions extracted from the CSV: {}",
        expense_tracker.transactions.len()
    );
    println!("Number of transactions ignored: {}", n_ignored_transactions);

    println!("{:?}", expense_tracker);

    Ok(())
}
